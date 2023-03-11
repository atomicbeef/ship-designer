use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::systems::init_colliders;
use common::colliders::{ShapeCollider, RegenerateColliders};
use common::player::Players;
use uflow::SendMode;

use common::colliders::{ColliderData, generate_collider_data};
use common::channels::Channel;
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::index::Index;
use common::network_id::NetworkId;
use common::packets::Packet;
use common::shape::{Shapes, ShapeHandle};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

pub fn spawn_shape(
    commands: &mut Commands,
    shapes: &Shapes,
    shape_handle: ShapeHandle,
    transform: Transform,
    shape_network_id: NetworkId,
    body: Entity,
) -> Entity {
    let shape = shapes.get(&shape_handle).unwrap();

    let shape_entity = commands.spawn(shape_handle)
        .insert(shape_network_id)
        .insert(TransformBundle::from_transform(transform))
        .id();
    
    commands.entity(body).add_child(shape_entity);

    let colliders = generate_collider_data(shape, transform);
    for collider_data in colliders {
        let collider_entity = commands.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(ShapeCollider::new(shape_entity))
            .id();
        commands.entity(body).add_child(collider_entity);
    }

    shape_entity
}

pub fn spawn_shape_exclusive(
    world: &mut World,
    shape_handle: ShapeHandle,
    transform: Transform,
    shape_network_id: NetworkId,
    body: Entity,
    colliders: Vec<ColliderData>
) -> Entity {
    let shape_entity = world.spawn(shape_handle)
        .insert(shape_network_id)
        .insert(TransformBundle::from_transform(transform))
        .id();
    
    (AddChild { parent: body, child: shape_entity }).write(world);
    
    for collider_data in colliders {
        let collider_entity = world.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(ShapeCollider::new(shape_entity))
            .id();
        (AddChild { parent: body, child: collider_entity }).write(world);
    }

    shape_entity
}

pub fn confirm_place_shape_requests(
    world: &mut World,
) {
    let place_shape_requests: Vec<PlaceShapeRequest> = world.get_resource_mut::<Events<PlaceShapeRequest>>()
        .unwrap()
        .drain()
        .collect();

    for place_shape_request in place_shape_requests {
        let body = world.get_resource::<Index<NetworkId>>().unwrap().entity(&place_shape_request.body_network_id).unwrap();
        let body_transform = world.get::<GlobalTransform>(body).unwrap();
        let shape_transform = Transform::from(place_shape_request.shape_transform);
        let (_, shape_global_rotation, shape_global_translation) = body_transform.mul_transform(shape_transform).to_scale_rotation_translation();

        let (shape_handle, shape_center) = {
            let shapes = world.get_resource::<Shapes>().unwrap();
            let shape_handle = shapes.get_handle(place_shape_request.shape_id);
            let shape_center = shapes.get(&shape_handle).unwrap().center();
            (shape_handle, shape_center)
        };

        // Prevent shapes from being placed inside of each other
        let shape_half_extents = shape_center - Vec3::splat(0.01);
        let rapier_context = world.get_resource::<RapierContext>().unwrap();

        if rapier_context.cast_shape(
            shape_global_translation,
            shape_global_rotation,
            Vec3::splat(0.0001),
            &Collider::cuboid(
                shape_half_extents.x,
                shape_half_extents.y,
                shape_half_extents.z
            ),
            0.01,
            QueryFilter::default()
        ).is_none() {
            let network_id = world.get_resource_mut::<NetworkIdGenerator>().unwrap().generate();
            let colliders = {
                let shapes = world.get_resource::<Shapes>().unwrap();
                let shape = shapes.get(&shape_handle).unwrap();
                generate_collider_data(shape, shape_transform)
            };

            spawn_shape_exclusive(world, shape_handle, shape_transform, network_id, body, colliders);

            let mut place_shape_events = world.get_resource_mut::<Events<PlaceShapeCommand>>().unwrap();
            place_shape_events.send(PlaceShapeCommand {
                shape_id: place_shape_request.shape_id,
                shape_network_id: network_id,
                transform: place_shape_request.shape_transform,
                body_network_id: place_shape_request.body_network_id
            });

            // Update colliders in Rapier
            SystemStage::single(init_colliders).run(world);
            world.resource_scope(|_, mut rapier_context: Mut<RapierContext>| {
                rapier_context.update_query_pipeline();
            });
        }
    }
}

pub fn confirm_delete_shape_requests(
    mut commands: Commands,
    mut delete_shape_request_reader: EventReader<DeleteShapeRequest>,
    mut send_delete_shape_writer: EventWriter<DeleteShapeCommand>,
    network_id_query: Query<(Entity, &NetworkId), With<ShapeHandle>>,
    ship_children_query: Query<&Children>,
    shape_collider_query: Query<&ShapeCollider>,
    parent_query: Query<&Parent>
) {
    for delete_shape_request in delete_shape_request_reader.iter() {
        for (shape_entity, network_id) in network_id_query.iter() {
            if *network_id == delete_shape_request.0 {
                let ship = parent_query.get(shape_entity).unwrap().get();

                // Remove colliders
                let children = ship_children_query.get(ship).unwrap();
                for &child in children {
                    if let Ok(shape_collider) = shape_collider_query.get(child) {
                        if shape_collider.shape == shape_entity {
                            commands.entity(ship).remove_children(&[child]);
                            commands.entity(child).despawn();
                        }
                    }
                }

                commands.entity(ship).remove_children(&[shape_entity]);
                commands.entity(shape_entity).despawn();
                send_delete_shape_writer.send(DeleteShapeCommand(delete_shape_request.0));
                break;
            }
        }
    }
}

pub fn send_place_shape_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_place_shape_reader: EventReader<PlaceShapeCommand>
) {
    for place_shape_command in send_place_shape_reader.iter() {
        let packet = Packet::from(place_shape_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

pub fn send_delete_shape_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_delete_shape_reader: EventReader<DeleteShapeCommand>
) {
    for delete_shape_command in send_delete_shape_reader.iter() {
        let packet = Packet::from(delete_shape_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

pub fn regenerate_colliders(
    mut commands: Commands,
    mut regenerate_colliders_reader: EventReader<RegenerateColliders>,
    parent_query: Query<&Parent, With<ShapeHandle>>,
    children_query: Query<&Children>,
    shape_colliders_query: Query<&ShapeCollider>,
    shape_query: Query<(&ShapeHandle, &Transform)>,
    shapes: Res<Shapes>
) {
    for request in regenerate_colliders_reader.iter() {
        let (shape_handle, transform) = shape_query.get(request.0).unwrap();
        let shape = shapes.get(&shape_handle).unwrap();

        if let Ok(parent) = parent_query.get(request.0) {
            let body = parent.get();

            // Delete old colliders
            for &child in children_query.get(body).unwrap() {
                if let Ok(shape_collider) = shape_colliders_query.get(child) {
                    if shape_collider.shape == request.0 {
                        commands.entity(body).remove_children(&[child]);
                        commands.entity(child).despawn();
                    }
                }
            }

            // Spawn new colliders
            let colliders = generate_collider_data(shape, *transform);
            for collider_data in colliders {
                let collider_entity = commands.spawn(collider_data.collider)
                    .insert(TransformBundle::from_transform(collider_data.transform))
                    .insert(ShapeCollider::new(request.0))
                    .id();
                commands.entity(body).add_child(collider_entity);
            }
        }
    }
}