use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_rapier3d::prelude::*;

use common::network_id::NetworkId;
use common::part::colliders::{RegenerateColliders, PartCollider};
use common::part::events::{VoxelUpdate, DeletePartCommand};
use common::part::materials::{Material, MaterialResistances};
use common::missile::{Missile, SpawnMissileRequest, SpawnMissileCommand, ExplodeMissileCommand, MissileBundle};
use common::player::PlayerId;
use packets::Packet;
use common::channels::Channel;
use common::part::{PartHandle, Parts, VOXEL_SIZE, DeletePart};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

fn spawn_missiles(
    mut spawn_request_reader: EventReader<SpawnMissileRequest>,
    mut spawn_command_writer: EventWriter<SpawnMissileCommand>,
    mut commands: Commands,
    mut network_id_generator: ResMut<NetworkIdGenerator>,
) {
    for spawn_event in spawn_request_reader.iter() {
        let network_id = network_id_generator.generate();

        commands.spawn(MissileBundle {
            missile: Missile::new(5.0),
            network_id,
            transform: TransformBundle::from_transform(spawn_event.transform.into()),
            velocity: Velocity::linear(spawn_event.velocity),
            collider: Collider::cuboid(0.25, 0.25, 0.25),
            ..Default::default()
        }).insert(ActiveEvents::COLLISION_EVENTS);

        spawn_command_writer.send(SpawnMissileCommand {
            transform: spawn_event.transform, 
            velocity: spawn_event.velocity,
            network_id
        });
    }
}

fn send_spawn_missile_commands(
    mut server_state: NonSendMut<ServerState>,
    player_id_query: Query<&PlayerId>,
    mut spawn_command_reader: EventReader<SpawnMissileCommand>
) {
    for spawn_command in spawn_command_reader.iter() {
        let packet = Packet::from(spawn_command);

        for &player_id in player_id_query.iter() {
            server_state.send_to_player(
                player_id,
                (&packet).into(),
                Channel::Missile.into(),
                uflow::SendMode::Reliable
            );
        }
    }
}

fn explode_missiles(
    rapier_context: Res<RapierContext>,
    material_resistances: Res<MaterialResistances>,
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sensor_query: Query<&Sensor>,
    missile_query: Query<&Missile>,
    part_collider_query: Query<&PartCollider>,
    mut part_query_set: ParamSet<(
        Query<(&GlobalTransform, &mut PartHandle)>,
        Query<&PartHandle>,
    )>,
    global_transform_query: Query<&GlobalTransform>,
    network_id_query: Query<&NetworkId>,
    mut parts: ResMut<Parts>,
    mut explode_missile_command_writer: EventWriter<ExplodeMissileCommand>,
    mut regenerate_colliders_writer: EventWriter<RegenerateColliders>,
    mut voxel_update_writer: EventWriter<VoxelUpdate>,
    mut delete_part_command_writer: EventWriter<DeletePartCommand>
) {
    let mut affected_parts = HashSet::new();
    let mut modified_parts = HashSet::new();
    let mut deleted_parts = HashSet::new();

    let mut exploded_missile_entities: HashSet<Entity> = HashSet::new();

    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(e1, e2, _) = collision_event {
            if exploded_missile_entities.contains(e1) || exploded_missile_entities.contains(e2) {
                continue;
            }

            if sensor_query.get(*e1).is_ok() == sensor_query.get(*e2).is_ok() {
                continue;
            }

            let (missile_entity, missile) = match missile_query.get(*e1) {
                Ok(missile) => (*e1, missile),
                Err(_) => match missile_query.get(*e2) {
                    Ok(missile) => (*e2, missile),
                    Err(_) => continue,
                },
            };

            affected_parts.extend(explode_missile(
                missile_entity,
                missile.power,
                &rapier_context,
                &material_resistances,
                &part_collider_query,
                &mut part_query_set.p0(),
                &global_transform_query,
                &mut parts,
            ));

            let network_id = network_id_query.get(missile_entity).copied().unwrap();
            explode_missile_command_writer.send(ExplodeMissileCommand { 
                network_id,
                transform: global_transform_query.get(missile_entity).copied().unwrap().into()
            });

            exploded_missile_entities.insert(missile_entity);
            commands.entity(missile_entity).despawn();
        }
    }

    let part_query = part_query_set.p1();

    for affected_part in affected_parts {
        let part_handle = part_query.get(affected_part).unwrap();
        let part = parts.get(part_handle).unwrap();
        let network_id = network_id_query.get(affected_part).copied().unwrap();
        
        if part.is_empty() {
            deleted_parts.insert((affected_part, network_id));
        } else {
            modified_parts.insert((affected_part, network_id, Vec::from(part.voxels())));
        }
    }

    for (entity, network_id, voxels) in modified_parts {
        regenerate_colliders_writer.send(RegenerateColliders(entity));
        voxel_update_writer.send(VoxelUpdate { network_id, voxels });
    }

    for (entity, network_id) in deleted_parts {
        commands.add(DeletePart(entity));

        delete_part_command_writer.send(DeletePartCommand(network_id));
    }
}

fn explode_missile(
    missile: Entity,
    missile_power: f32,
    rapier_context: &RapierContext,
    material_resistances: &MaterialResistances,
    part_collider_query: &Query<&PartCollider>,
    voxel_intersection_query: &mut Query<(&GlobalTransform, &mut PartHandle)>,
    global_transform_query: &Query<&GlobalTransform>,
    parts: &mut Parts
) -> HashSet<Entity> {
    let missile_pos = global_transform_query.get(missile).unwrap().translation();

    let mut affected_parts = HashSet::new();

    let points = 5000;
    for i in 0..points {
        // Add an offset to each point to optimize for nearest neighbor distance
        let offset_point = i as f32 + 0.36;

        let angle_increment = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let incline = (1.0 - 2.0 * (offset_point / points as f32)).acos();
        let azimuth = TAU * angle_increment * offset_point;

        let direction = Vec3::new(
            incline.sin() * azimuth.sin(),
            incline.cos(),
            incline.sin() * azimuth.cos()
        ).normalize();

        let mut origin = missile_pos;
        let mut power = missile_power;

        loop {
            if let Some((entity, toi)) = rapier_context.cast_ray(
                origin,
                direction,
                power,
                true,
                QueryFilter::default().exclude_sensors().predicate(
                    &|entity| part_collider_query.get(entity).is_ok()
                )
            ) {
                let part_entity = part_collider_query.get(entity).unwrap().part;

                if let Ok((&part_transform, mut part_handle)) = voxel_intersection_query.get_mut(part_entity) {
                    let part = parts.get_mut(&mut part_handle).unwrap();
                    let part_intersection_pos = origin + direction * toi;

                    power -= toi;

                    let inverse = part_transform.affine().inverse();

                    let mut ray_pos_part = (inverse.transform_point3(part_intersection_pos) + part.center()) / VOXEL_SIZE;
                    let part_space_direction = inverse.transform_vector3(direction).normalize();

                    while power > 0.0 && part.voxel_is_in_part(ray_pos_part.into()) {
                        let material = part.get(ray_pos_part.into());

                        power -= VOXEL_SIZE + material_resistances.get(material);

                        if power > 0.0 {
                            part.set(ray_pos_part.into(), Material::Empty);

                            affected_parts.insert(part_entity);
                        }

                        ray_pos_part += part_space_direction * VOXEL_SIZE;
                    }

                    if power <= 0.0 {
                        break;
                    }

                    let ray_pos_world = part_transform.transform_point(ray_pos_part * VOXEL_SIZE - part.center());
                    // Add VOXEL_SIZE / 2.0 to make sure we don't hit the same part next time
                    origin = ray_pos_world + direction * VOXEL_SIZE / 2.0;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    affected_parts
}

fn send_explode_missile_commands(
    mut server_state: NonSendMut<ServerState>,
    player_id_query: Query<&PlayerId>,
    mut explode_missile_command_reader: EventReader<ExplodeMissileCommand>,
) {
    for explode_missile in explode_missile_command_reader.iter() {
        let packet = Packet::from(explode_missile);

        for &player_id in player_id_query.iter() {
            server_state.send_to_player(
                player_id,
                (&packet).into(),
                Channel::Missile.into(),
                uflow::SendMode::Reliable
            );
        }
    }
}

pub struct ServerMissilePlugin;

impl Plugin for ServerMissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            spawn_missiles,
            send_spawn_missile_commands.after(spawn_missiles),
            explode_missiles,
            send_explode_missile_commands.after(explode_missiles),
        ).in_schedule(CoreSchedule::FixedUpdate));
    }
}