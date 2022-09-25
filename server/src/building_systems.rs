use bevy::prelude::*;
use common::packets::Packet;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlaceBlockRequest, PlaceBlockCommand, DeleteBlockRequest, DeleteBlockCommand};
use common::grid::Grid;

use crate::server_state::ServerState;

pub fn confirm_place_block_requests(
    mut grid_query: Query<&mut Grid>,
    mut place_block_request_reader: EventReader<PlaceBlockRequest>,
    mut send_place_block_writer: EventWriter<PlaceBlockCommand>,
    mut commands: Commands,
){
    let mut grid = grid_query.single_mut();

    for place_block_request in place_block_request_reader.iter() {
        if !grid.exists_at(&place_block_request.0) {
            grid.set(&place_block_request.0, Some(commands.spawn().id()));
            send_place_block_writer.send(PlaceBlockCommand(place_block_request.0));
        }
    }
}

pub fn confirm_delete_block_requests(
    mut grid_query: Query<&mut Grid>,
    mut delete_block_request_reader: EventReader<DeleteBlockRequest>,
    mut send_delete_block_writer: EventWriter<DeleteBlockCommand>
){
    let mut grid = grid_query.single_mut();

    for delete_block_request in delete_block_request_reader.iter() {
        if grid.exists_at(&delete_block_request.0) {
            grid.set(&delete_block_request.0, None);
            send_delete_block_writer.send(DeleteBlockCommand(delete_block_request.0));
        }
    }
}

pub fn send_place_block_commands(
    mut server_state: ResMut<ServerState>,
    mut send_place_block_reader: EventReader<PlaceBlockCommand>
) {
    for place_block_command in send_place_block_reader.iter() {
        for peer in server_state.peers_mut() {
            let packet: Packet = place_block_command.into();
            peer.send((&packet).into(), Channel::BlockCommands.into(), SendMode::Reliable);
        }
    }
}

pub fn send_delete_block_commands(
    mut server_state: ResMut<ServerState>,
    mut send_delete_block_reader: EventReader<DeleteBlockCommand>
) {
    for delete_block_command in send_delete_block_reader.iter() {
        for peer in server_state.peers_mut() {
            let packet: Packet = delete_block_command.into();
            peer.send((&packet).into(), Channel::BlockCommands.into(), SendMode::Reliable);
        }
    }
}