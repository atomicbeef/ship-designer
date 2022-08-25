use bevy::prelude::*;
use uflow::Event::*;

use common::events::{PlaceBlockRequest, DeleteBlockRequest};
use common::packets::{Packet, PacketType};

use crate::server_state::ServerState;

pub fn process_packets(
    mut state: ResMut<ServerState>,
    mut place_block_request_writer: EventWriter<PlaceBlockRequest>,
    mut delete_block_request_writer: EventWriter<DeleteBlockRequest>
) {
    state.server.step();

    for new_peer in state.server.incoming() {
        info!("New incoming connection from {}", new_peer.address());
        state.peer_list.push(new_peer);
    }

    for peer in state.peer_list.iter_mut() {
        for event in peer.poll_events() {
            match event {
                Connect => {
                    info!("{} connected", peer.address());
                },

                Disconnect => {
                    info!("{} disconnected", peer.address());
                },

                Timeout => {
                    info!("{} timed out", peer.address());
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

    state.peer_list.retain(|peer| !peer.is_disconnected());
}

fn generate_events(
    packet: Packet,
    place_block_writer: &mut EventWriter<PlaceBlockRequest>,
    delete_block_writer: &mut EventWriter<DeleteBlockRequest>
) {
    match packet.packet_type() {
        PacketType::PlaceBlock => {
            match PlaceBlockRequest::try_from(packet) {
                Ok(place_block_request) => {
                    place_block_writer.send(place_block_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::DeleteBlock => {
            match DeleteBlockRequest::try_from(packet) {
                Ok(delete_block_request) => {
                    delete_block_writer.send(delete_block_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        }
    }
}