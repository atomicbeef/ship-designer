use bevy::prelude::*;
use uflow::Event::*;

use common::events::building::{PlaceShapeRequest, DeleteShapeRequest};
use common::events::player_connection::{PlayerConnected, PlayerDisconnected};
use common::packets::{Packet, PacketType};

use crate::server_state::ServerState;

pub fn process_packets(
    mut state: ResMut<ServerState>,
    mut place_block_request_writer: EventWriter<PlaceShapeRequest>,
    mut delete_block_request_writer: EventWriter<DeleteShapeRequest>,
    mut client_connected_writer: EventWriter<PlayerConnected>,
    mut client_disconnected_writer: EventWriter<PlayerDisconnected>
) {
    state.server.step();

    for new_peer in state.server.incoming() {
        info!("New incoming connection from {}", new_peer.address());
        state.add_player(new_peer, "Test".to_string());
    }

    for (player, peer) in state.players_peers_mut() {
        for event in peer.poll_events() {
            match event {
                Connect => {
                    info!("{} connected from {}", player.name(), peer.address());
                    let player_connected_event = PlayerConnected { id: player.id(), name: player.name().clone() };
                    client_connected_writer.send(player_connected_event);
                },

                Disconnect => {
                    info!("{} disconnected", player.name());
                    client_disconnected_writer.send(PlayerDisconnected(player.id()));
                },

                Timeout => {
                    info!("{} timed out", peer.address());
                    client_disconnected_writer.send(PlayerDisconnected(player.id()));
                },

                Receive(packet_data) => {
                    match Packet::try_from(packet_data) {
                        Ok(packet) => {
                            debug!("Received packet {:?}", packet);
                            generate_events(packet, &mut place_block_request_writer, &mut delete_block_request_writer);
                        },
                        Err(err) => {
                            warn!(?err);
                        }
                    };
                }
            }
        }
    }

    state.server.flush();
}

fn generate_events(
    packet: Packet,
    place_block_writer: &mut EventWriter<PlaceShapeRequest>,
    delete_block_writer: &mut EventWriter<DeleteShapeRequest>
) {
    match packet.packet_type() {
        PacketType::PlaceShape => {
            match PlaceShapeRequest::try_from(packet) {
                Ok(place_block_request) => {
                    place_block_writer.send(place_block_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::DeleteShape => {
            match DeleteShapeRequest::try_from(packet) {
                Ok(delete_block_request) => {
                    delete_block_writer.send(delete_block_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::InitialState => {},
        PacketType::PlayerConnected => {},
        PacketType::PlayerDisconnected => {}
    }
}