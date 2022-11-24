use bevy::prelude::*;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlayerConnected, PlayerDisconnected, InitialState};
use common::grid::GridPos;
use common::network_id::NetworkId;
use common::shape::ShapeHandle;
use common::packets::Packet;
use common::player::Player;

use crate::server_state::ServerState;

pub fn send_player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut server_state: ResMut<ServerState>,
    shape_query: Query<(&ShapeHandle, &Transform, &NetworkId)>
) {
    for player_connected in player_connected_reader.iter() {
        // Send new player connected packet to existing players
        let player_connected_packet = Packet::from(player_connected);
        
        for peer in server_state.peers_mut() {
            peer.send(
                (&player_connected_packet).into(),
                Channel::PlayerConnectionEvents.into(),
                SendMode::Reliable
            );
        }

        // Send the current state of the world to the new player
        let players: Vec<Player> = server_state.players().cloned().collect();

        let mut shapes: Vec<(ShapeHandle, GridPos, NetworkId)> = Vec::new();
        for (shape_handle, transform, network_id) in shape_query.iter() {
            let grid_pos = GridPos::from(transform);
            shapes.push((shape_handle.clone(), grid_pos, *network_id));
        }

        let initial_state = InitialState { players, shapes };

        let initial_state_packet = Packet::from(&initial_state);

        if let Some(connected_peer) = server_state.peer_mut(player_connected.id) {
            connected_peer.send(
                (&initial_state_packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

pub fn send_player_disconnected(
    mut player_disconnected_reader: EventReader<PlayerDisconnected>,
    mut server_state: ResMut<ServerState>
) {
    for disconnected_player in player_disconnected_reader.iter() {
        server_state.remove_player(disconnected_player.0);

        let packet = Packet::from(disconnected_player);

        for peer in server_state.peers_mut() {
            peer.send((&packet).into(), Channel::PlayerConnectionEvents.into(), SendMode::Reliable);
        }
    }
}