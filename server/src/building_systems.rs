use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::prelude::systems::init_colliders;
use common::player::Players;
use uflow::SendMode;

use common::channels::Channel;
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::network_id::{NetworkId, NetworkIdIndex,};
use common::packets::Packet;
use common::shape::{Shapes, ShapeHandle};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

pub fn spawn_shape(
    commands: &mut Commands,
    shape_handle: ShapeHandle,
    transform: TransformBundle,
    shape_network_id: NetworkId,
    shapes: &Shapes,
    body: Entity
) -> Entity {
    let shape = shapes.get(&shape_handle).unwrap();
    let shape_half_extents = shape.center();

    let shape_entity = commands.spawn(shape_handle)
        .insert(shape_network_id)
        .insert(transform)
        .insert(Collider::cuboid(
            shape_half_extents.x, 
            shape_half_extents.y,
            shape_half_extents.z
        ))
        .id();
    
    commands.entity(body).add_child(shape_entity);

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
        let body = world.get_resource::<NetworkIdIndex>().unwrap().entity(&place_shape_request.body_network_id).unwrap();
        let shape_transform = Transform::from(place_shape_request.shape_transform);

        let shape_handle;
        let shape_center;

        {
            let shapes = world.get_resource::<Shapes>().unwrap();
            shape_handle = shapes.get_handle(place_shape_request.shape_id);
            shape_center = shapes.get(&shape_handle).unwrap().center();
        }

        // Prevent shapes from being placed inside of each other
        let shape_half_extents = shape_center - Vec3::splat(0.01);
        let rapier_context = world.get_resource::<RapierContext>().unwrap();

        if rapier_context.cast_shape(
            shape_transform.translation,
            shape_transform.rotation,
            Vec3::splat(0.0001),
            &Collider::cuboid(
                shape_half_extents.x,
                shape_half_extents.y,
                shape_half_extents.z
            ),
            0.01,
            QueryFilter::default()
        ).is_none() {
            let shape_handle = world.get_resource::<Shapes>().unwrap().get_handle(place_shape_request.shape_id);
            let network_id = world.get_resource_mut::<NetworkIdGenerator>().unwrap().generate();

            let shape_entity = world.spawn(shape_handle)
                .insert(network_id)
                .insert(shape_transform)
                .insert(Collider::cuboid(
                    shape_center.x,
                    shape_center.y,
                    shape_center.z
                ))
                .id();
        
            (AddChild { parent: body, child: shape_entity }).write(world);

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
    parent_query: Query<&Parent>
) {
    for delete_shape_request in delete_shape_request_reader.iter() {
        for (entity, network_id) in network_id_query.iter() {
            if *network_id == delete_shape_request.0 {
                let ship = parent_query.get(entity).unwrap().get();
                commands.entity(ship).remove_children(&[entity]);
                commands.entity(entity).despawn();
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