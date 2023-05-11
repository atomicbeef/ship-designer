use bevy::prelude::*;
use common::entity_lookup::lookup;
use common::missile::SpawnMissileRequest;
use uflow::server::Event::*;
use uflow::server::ErrorType;

use common::part::events::{PlacePartRequest, DeletePartRequest};
use common::player_connection::{PlayerConnected, PlayerDisconnected};
use packets::{Packet, PacketType};
use common::player::{PlayerId, PlayerName};

use crate::server_state::ServerState;

pub fn process_packets(
    mut state: NonSendMut<ServerState>,
    mut commands: Commands,
    player_entity_query: Query<(Entity, &PlayerId)>,
    player_name_query: Query<&PlayerName>,
    mut place_part_request_writer: EventWriter<PlacePartRequest>,
    mut delete_part_request_writer: EventWriter<DeletePartRequest>,
    mut client_connected_writer: EventWriter<PlayerConnected>,
    mut client_disconnected_writer: EventWriter<PlayerDisconnected>,
    mut spawn_missile_writer: EventWriter<SpawnMissileRequest>,
) {
    state.server.flush();

    for event in state.server.step() {
        match event {
            Connect(address) => {
                info!("New incoming connection from {}", address);

                let player_id = state.new_player_id();
                state.add_client_address(player_id, address);

                let player_name = PlayerName::from("Player".to_string());

                client_connected_writer.send(PlayerConnected {
                    id: player_id,
                    name: player_name.clone(),
                    pos: Vec3::splat(30.0),
                });

                commands.spawn(player_id).insert(player_name);
            },
            Disconnect(address) => {
                if let Some(player_id) = state.player_id(address).cloned() {
                    if let Some(entity) = lookup(&player_entity_query, &player_id) {
                        let name = player_name_query.get(entity).unwrap();
                        info!("{} disconnected", name);
                        state.remove_client_address(player_id);
                        commands.entity(entity).despawn();

                        client_disconnected_writer.send(PlayerDisconnected(player_id));
                    }
                }
            },
            Receive(_, data) => {
                match Packet::try_from(data) {
                    Ok(packet) => {
                        debug!("Received packet {:?}", packet);
                        generate_events(
                            packet,
                            &mut place_part_request_writer,
                            &mut delete_part_request_writer,
                            &mut spawn_missile_writer,
                        );
                    },
                    Err(err) => {
                        warn!(?err);
                    }
                };
            },
            Error(address, err) => {
                match err {
                    ErrorType::Timeout => {
                        if let Some(player_id) = state.player_id(address).cloned() {
                            if let Some(entity) = lookup(&player_entity_query, &player_id) {
                                let name = player_name_query.get(entity).unwrap();
                                error!("{} timed out", name);
                                state.remove_client_address(player_id);
                                commands.entity(entity).despawn();

                                client_disconnected_writer.send(PlayerDisconnected(player_id));
                            }
                        }
                    },
                    ErrorType::Config => {
                        error!("{} failed to connect: endpoint configuration mismatch", address);
                    },
                    ErrorType::Version => {
                        error!("{} failed to connect: version mismatch", address);
                    },
                    ErrorType::ServerFull => {
                        error!("{} failed to connect: server full", address);
                    }
                }
            }
        }
    }
}

fn generate_events(
    packet: Packet,
    place_part_writer: &mut EventWriter<PlacePartRequest>,
    delete_part_writer: &mut EventWriter<DeletePartRequest>,
    spawn_missile_writer: &mut EventWriter<SpawnMissileRequest>,
) {
    match packet.packet_type() {
        PacketType::PlacePart => {
            match PlacePartRequest::try_from(packet) {
                Ok(place_part_request) => {
                    place_part_writer.send(place_part_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::DeletePart => {
            match DeletePartRequest::try_from(packet) {
                Ok(delete_part_request) => {
                    delete_part_writer.send(delete_part_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::InitialState => {},
        PacketType::PlayerConnected => {},
        PacketType::PlayerDisconnected => {},
        PacketType::VoxelUpdate => {},
        PacketType::SpawnMissile => {
            match SpawnMissileRequest::try_from(packet) {
                Ok(spawn_missile_request) => {
                    spawn_missile_writer.send(spawn_missile_request);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::ExplodeMissile => {},
    }
}