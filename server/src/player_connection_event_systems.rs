use bevy::prelude::*;
use uflow::SendMode;

use common::channels::Channel;
use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::network_id::NetworkId;
use common::shape::{ShapeHandle, Shapes, ShapeNetworkRepr};
use common::shape_transform::ShapeTransform;
use common::packets::Packet;
use common::player::Player;

use crate::server_state::ServerState;

pub fn send_player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut server_state: ResMut<ServerState>,
    shapes: Res<Shapes>,
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

        let mut shape_data: Vec<(ShapeNetworkRepr, ShapeTransform, NetworkId)> = Vec::new();
        for (shape_handle, transform, network_id) in shape_query.iter() {
            let shape_network_repr = match shapes.get(shape_handle) {
                Some(shape) => match shape.parent_shape_id() {
                    Some(_) => ShapeNetworkRepr::Child(shape.clone()),
                    None => ShapeNetworkRepr::Predefined(shape_handle.id()),
                },
                None => {
                    warn!("Attempted to send non-existent shape with ID {:?} to new player!", shape_handle.id());
                    continue;
                }
            };

            let shape_transform = ShapeTransform::from_xyz(
                transform.translation.x as i16,
                transform.translation.y as i16,
                transform.translation.z as i16
            );

            shape_data.push((shape_network_repr, shape_transform, *network_id));
        }

        let initial_state = InitialState { players, shapes: shape_data };

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