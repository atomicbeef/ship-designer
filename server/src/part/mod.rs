use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::systems::init_colliders;
use common::part::colliders::{PartCollider, RegenerateColliders};
use common::player::Players;
use uflow::SendMode;

use common::part::colliders::{ColliderData, generate_collider_data};
use common::channels::Channel;
use common::part::events::{PlacePartRequest, PlacePartCommand, DeletePartRequest, DeletePartCommand};
use common::index::Index;
use common::network_id::NetworkId;
use common::packets::Packet;
use common::part::{Parts, PartHandle};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

pub fn spawn_part(
    commands: &mut Commands,
    parts: &Parts,
    part_handle: PartHandle,
    transform: Transform,
    part_network_id: NetworkId,
    construct: Entity,
) -> Entity {
    let part = parts.get(&part_handle).unwrap();

    let part_entity = commands.spawn(part_handle)
        .insert(part_network_id)
        .insert(TransformBundle::from_transform(transform))
        .id();
    
    commands.entity(construct).add_child(part_entity);

    let colliders = generate_collider_data(part, transform);
    for collider_data in colliders {
        let collider_entity = commands.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(PartCollider::new(part_entity))
            .id();
        commands.entity(construct).add_child(collider_entity);
    }

    part_entity
}

pub fn spawn_part_exclusive(
    world: &mut World,
    part_handle: PartHandle,
    transform: Transform,
    part_network_id: NetworkId,
    construct: Entity,
    colliders: Vec<ColliderData>
) -> Entity {
    let part_entity = world.spawn(part_handle)
        .insert(part_network_id)
        .insert(TransformBundle::from_transform(transform))
        .id();
    
    (AddChild { parent: construct, child: part_entity }).write(world);
    
    for collider_data in colliders {
        let collider_entity = world.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(PartCollider::new(part_entity))
            .id();
        (AddChild { parent: construct, child: collider_entity }).write(world);
    }

    part_entity
}

fn confirm_place_part_requests(
    world: &mut World,
) {
    let place_part_requests: Vec<PlacePartRequest> = world.get_resource_mut::<Events<PlacePartRequest>>()
        .unwrap()
        .drain()
        .collect();

    for place_part_request in place_part_requests {
        let construct = world.get_resource::<Index<NetworkId>>().unwrap().entity(&place_part_request.construct_network_id).unwrap();
        let construct_transform = world.get::<GlobalTransform>(construct).unwrap();
        let part_transform = Transform::from(place_part_request.part_transform);
        let (_, part_global_rotation, part_global_translation) = construct_transform.mul_transform(part_transform).to_scale_rotation_translation();

        let (part_handle, part_center) = {
            let parts = world.get_resource::<Parts>().unwrap();
            let part_handle = parts.get_handle(place_part_request.part_id);
            let part_center = parts.get(&part_handle).unwrap().center();
            (part_handle, part_center)
        };

        // Prevent parts from being placed inside of each other
        let part_half_extents = part_center - Vec3::splat(0.01);
        let rapier_context = world.get_resource::<RapierContext>().unwrap();

        if rapier_context.cast_shape(
            part_global_translation,
            part_global_rotation,
            Vec3::splat(0.0001),
            &Collider::cuboid(
                part_half_extents.x,
                part_half_extents.y,
                part_half_extents.z
            ),
            0.01,
            QueryFilter::default()
        ).is_none() {
            let network_id = world.get_resource_mut::<NetworkIdGenerator>().unwrap().generate();
            let colliders = {
                let parts = world.get_resource::<Parts>().unwrap();
                let part = parts.get(&part_handle).unwrap();
                generate_collider_data(part, part_transform)
            };

            spawn_part_exclusive(world, part_handle, part_transform, network_id, construct, colliders);

            let mut place_part_events = world.get_resource_mut::<Events<PlacePartCommand>>().unwrap();
            place_part_events.send(PlacePartCommand {
                part_id: place_part_request.part_id,
                part_network_id: network_id,
                transform: place_part_request.part_transform,
                construct_network_id: place_part_request.construct_network_id
            });

            // Update colliders in Rapier
            Schedule::new().add_system(init_colliders).run(world);
            world.resource_scope(|_, mut rapier_context: Mut<RapierContext>| {
                rapier_context.update_query_pipeline();
            });
        }
    }
}

fn confirm_delete_part_requests(
    mut commands: Commands,
    mut delete_part_request_reader: EventReader<DeletePartRequest>,
    mut send_delete_part_writer: EventWriter<DeletePartCommand>,
    network_id_query: Query<(Entity, &NetworkId), With<PartHandle>>,
    construct_children_query: Query<&Children>,
    part_collider_query: Query<&PartCollider>,
    parent_query: Query<&Parent>
) {
    for delete_part_request in delete_part_request_reader.iter() {
        for (part_entity, network_id) in network_id_query.iter() {
            if *network_id == delete_part_request.0 {
                let construct = parent_query.get(part_entity).unwrap().get();

                // Remove colliders
                let children = construct_children_query.get(construct).unwrap();
                for &child in children {
                    if let Ok(part_collider) = part_collider_query.get(child) {
                        if part_collider.part == part_entity {
                            commands.entity(construct).remove_children(&[child]);
                            commands.entity(child).despawn();
                        }
                    }
                }

                commands.entity(construct).remove_children(&[part_entity]);
                commands.entity(part_entity).despawn();
                send_delete_part_writer.send(DeletePartCommand(delete_part_request.0));
                break;
            }
        }
    }
}

fn send_place_part_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_place_part_reader: EventReader<PlacePartCommand>
) {
    for place_part_command in send_place_part_reader.iter() {
        let packet = Packet::from(place_part_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::PartCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

fn send_delete_part_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_delete_part_reader: EventReader<DeletePartCommand>
) {
    for delete_part_command in send_delete_part_reader.iter() {
        let packet = Packet::from(delete_part_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::PartCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

fn regenerate_colliders(
    mut commands: Commands,
    mut regenerate_colliders_reader: EventReader<RegenerateColliders>,
    parent_query: Query<&Parent, With<PartHandle>>,
    children_query: Query<&Children>,
    part_colliders_query: Query<&PartCollider>,
    part_query: Query<(&PartHandle, &Transform)>,
    parts: Res<Parts>
) {
    for request in regenerate_colliders_reader.iter() {
        let (part_handle, transform) = part_query.get(request.0).unwrap();
        let part = parts.get(&part_handle).unwrap();

        if let Ok(parent) = parent_query.get(request.0) {
            let construct = parent.get();

            // Delete old colliders
            for &child in children_query.get(construct).unwrap() {
                if let Ok(part_collider) = part_colliders_query.get(child) {
                    if part_collider.part == request.0 {
                        commands.entity(construct).remove_children(&[child]);
                        commands.entity(child).despawn();
                    }
                }
            }

            // Spawn new colliders
            let colliders = generate_collider_data(part, *transform);
            for collider_data in colliders {
                let collider_entity = commands.spawn(collider_data.collider)
                    .insert(TransformBundle::from_transform(collider_data.transform))
                    .insert(PartCollider::new(request.0))
                    .id();
                commands.entity(construct).add_child(collider_entity);
            }
        }
    }
}

pub struct ServerPartPlugin;
impl Plugin for ServerPartPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(confirm_place_part_requests)
            .add_system(confirm_delete_part_requests)
            .add_system(send_place_part_commands)
            .add_system(send_delete_part_commands)
            .add_system(regenerate_colliders);
    }
}