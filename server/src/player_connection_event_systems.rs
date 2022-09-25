use bevy::prelude::*;
use common::grid::Grid;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlayerConnected, PlayerDisconnected, InitialState};
use common::packets::Packet;
use common::player::Player;

use crate::server_state::ServerState;

pub fn send_player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut server_state: ResMut<ServerState>,
    grid_query: Query<&Grid>
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

        // Send the current state of the grid to the new player
        if let Ok(grid) = grid_query.get_single() {
            let players: Vec<Player> = server_state.players().cloned().collect();
            let grid_positions = grid.positions();
            let initial_state = InitialState { players, grid_positions };

            let initial_state_packet = Packet::from(&initial_state);

            if let Some(connected_peer) = server_state.peer_mut(player_connected.id) {
                connected_peer.send(
                    (&initial_state_packet).into(),
                    Channel::BlockCommands.into(),
                    SendMode::Reliable
                );
            }
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