use bevy::prelude::*;
use common::grid::GridPos;
use common::packets::Packet;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::network_id::NetworkId;
use common::shape::{ShapeHandle, ShapeHandleType};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

pub fn spawn_shape(
    commands: &mut Commands,
    shape_handle: ShapeHandle,
    transform: Transform,
    network_id: NetworkId
) -> Entity {
    commands.spawn()
        .insert(shape_handle)
        .insert(network_id)
        .insert(transform)
        .id()
}

pub fn confirm_place_shape_requests(
    mut place_shape_request_reader: EventReader<PlaceShapeRequest>,
    mut send_place_shape_writer: EventWriter<PlaceShapeCommand>,
    mut commands: Commands,
    mut network_id_generator: ResMut<NetworkIdGenerator>,
    transform_query: Query<&Transform, With<ShapeHandle>>
) {
    'requests: for place_shape_request in place_shape_request_reader.iter() {
        // Prevent shapes from being placed inside of each other
        for transform in transform_query.iter() {
            let grid_pos = GridPos::from(transform);
            if grid_pos == place_shape_request.1 {
                continue 'requests;
            }
        }

        // Spawn shape
        let network_id = network_id_generator.generate();
        let shape_handle = ShapeHandle::new(place_shape_request.0, ShapeHandleType::ReadOnly);
        spawn_shape(&mut commands,  shape_handle, Transform::from(place_shape_request.1), network_id);

        send_place_shape_writer.send(PlaceShapeCommand {
            shape_handle_id: place_shape_request.0,
            network_id,
            pos: place_shape_request.1
        });
    }
}

pub fn confirm_delete_shape_requests(
    mut commands: Commands,
    mut delete_shape_request_reader: EventReader<DeleteShapeRequest>,
    mut send_delete_shape_writer: EventWriter<DeleteShapeCommand>,
    network_id_query: Query<(Entity, &NetworkId), With<ShapeHandle>>
) {
    for delete_shape_request in delete_shape_request_reader.iter() {
        for (entity, network_id) in network_id_query.iter() {
            if *network_id == delete_shape_request.0 {
                commands.entity(entity).despawn();
                send_delete_shape_writer.send(DeleteShapeCommand(delete_shape_request.0));
                break;
            }
        }
    }
}

pub fn send_place_shape_commands(
    mut server_state: ResMut<ServerState>,
    mut send_place_shape_reader: EventReader<PlaceShapeCommand>
) {
    for place_shape_command in send_place_shape_reader.iter() {
        for peer in server_state.peers_mut() {
            let packet: Packet = place_shape_command.into();
            peer.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
        }
    }
}

pub fn send_delete_shape_commands(
    mut server_state: ResMut<ServerState>,
    mut send_delete_shape_reader: EventReader<DeleteShapeCommand>
) {
    for delete_shape_command in send_delete_shape_reader.iter() {
        for peer in server_state.peers_mut() {
            let packet: Packet = delete_shape_command.into();
            peer.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
        }
    }
}