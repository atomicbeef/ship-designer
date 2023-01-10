use bevy::prelude::*;
use common::ship::Ship;
use uflow::SendMode;

use common::channels::Channel;
use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::network_id::NetworkId;
use common::shape::{ShapeHandle, Shapes, ShapeNetworkRepr};
use common::shape_transform::ShapeTransform;
use common::packets::Packet;
use common::player::{Player, Players};

use crate::server_state::ServerState;

pub fn send_player_connected(
    ship_query: Query<&NetworkId, &Ship>,
    mut player_connected_reader: EventReader<PlayerConnected>,
    players: Res<Players>,
    mut server_state: NonSendMut<ServerState>,
    shapes: Res<Shapes>,
    shape_query: Query<(&ShapeHandle, &Transform, &NetworkId)>
) {
    for ship_network_id in ship_query.iter() {
        for player_connected in player_connected_reader.iter() {
            // Send new player connected packet to existing players
            let player_connected_packet = Packet::from(player_connected);
            
            for player_id in players.ids() {
                server_state.send_to_player(
                    *player_id,
                    (&player_connected_packet).into(),
                    Channel::PlayerConnectionEvents.into(),
                    SendMode::Reliable
                );
            }

            // Send the current state of the world to the new player
            let players: Vec<Player> = players.players().cloned().collect();

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

            let initial_state = InitialState {
                players,
                body_network_id: *ship_network_id,
                shapes: shape_data
            };
            let initial_state_packet = Packet::from(&initial_state);
            
            server_state.send_to_player(
                player_connected.id,
                (&initial_state_packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

pub fn send_player_disconnected(
    mut player_disconnected_reader: EventReader<PlayerDisconnected>,
    players: Res<Players>,
    mut server_state: NonSendMut<ServerState>
) {
    for disconnected_player in player_disconnected_reader.iter() {
        let packet = Packet::from(disconnected_player);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::PlayerConnectionEvents.into(),
                SendMode::Reliable
            );
        }
    }
}