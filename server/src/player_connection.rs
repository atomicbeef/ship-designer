use bevy::prelude::*;
use common::ship::Ship;
use uflow::SendMode;

use common::channels::Channel;
use common::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::network_id::NetworkId;
use common::part::{PartHandle, Parts, PartNetworkRepr};
use common::compact_transform::CompactTransform;
use common::packets::Packet;
use common::player::{Player, Players};

use crate::server_state::ServerState;

fn send_player_connected(
    ship_query: Query<(Entity, &NetworkId, &Transform), &Ship>,
    mut player_connected_reader: EventReader<PlayerConnected>,
    players: Res<Players>,
    mut server_state: NonSendMut<ServerState>,
    parts: Res<Parts>,
    part_query: Query<(&PartHandle, &Transform, &NetworkId)>,
    ship_children_query: Query<&Children>,
) {
    for (ship, ship_network_id, ship_transform) in ship_query.iter() {
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

            let mut part_data: Vec<(PartNetworkRepr, CompactTransform, NetworkId)> = Vec::new();
            let ship_children = ship_children_query.get(ship).unwrap();
            for &child in ship_children {
                if let Ok((part_handle, transform, network_id)) = part_query.get(child) {
                    let part_network_repr = match parts.get(part_handle) {
                        Some(part) => match part.parent_part_id() {
                            Some(_) => PartNetworkRepr::Child(part.clone()),
                            None => PartNetworkRepr::Predefined(part_handle.id()),
                        },
                        None => {
                            warn!("Attempted to send non-existent part with ID {:?} to new player!", part_handle.id());
                            continue;
                        }
                    };
    
                    let part_transform = CompactTransform::from(*transform);
    
                    part_data.push((part_network_repr, part_transform, *network_id));
                }
            }

            let initial_state = InitialState {
                players,
                construct_network_id: *ship_network_id,
                parts: part_data,
                construct_transform: CompactTransform::from(*ship_transform)
            };
            let initial_state_packet = Packet::from(&initial_state);
            
            info!("Sending initial state to {:?}", player_connected.id);

            server_state.send_to_player(
                player_connected.id,
                (&initial_state_packet).into(),
                Channel::PartCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

fn send_player_disconnected(
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

pub struct PlayerConnectionPlugin;

impl Plugin for PlayerConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerConnected>()
            .add_event::<PlayerDisconnected>()
            .add_system(send_player_connected)
            .add_system(send_player_disconnected);
    }
}