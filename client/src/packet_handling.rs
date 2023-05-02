use bevy::app::AppExit;
use bevy::prelude::*;
use common::missile::{SpawnMissileCommand, ExplodeMissileCommand};
use common::part::events::VoxelUpdate;
use uflow::client::{Event::*, ErrorType};

use common::part::events::{PlacePartCommand, DeletePartCommand};
use common::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use packets::{Packet, PacketType};

use crate::connection_state::ConnectionState;

pub fn process_packets(
    mut state: ResMut<ConnectionState>,
    mut place_part_command_writer: EventWriter<PlacePartCommand>,
    mut delete_part_command_writer: EventWriter<DeletePartCommand>,
    mut player_connected_writer: EventWriter<PlayerConnected>,
    mut player_disconnected_writer: EventWriter<PlayerDisconnected>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut initial_state_writer: EventWriter<InitialState>,
    mut voxel_update_writer: EventWriter<VoxelUpdate>,
    mut spawn_missile_writer: EventWriter<SpawnMissileCommand>,
    mut explode_missile_writer: EventWriter<ExplodeMissileCommand>,
) {
    for event in state.client.step() {
        match event {
            Connect => {
                info!("Connected to server");
            },
            Disconnect => {
                info!("Disconnected from server");
                app_exit_writer.send(AppExit);
            },
            Receive(packet_data) => {
                match Packet::try_from(packet_data) {
                    Ok(packet) => {
                        generate_events(
                            packet,
                            &mut place_part_command_writer,
                            &mut delete_part_command_writer,
                            &mut player_connected_writer,
                            &mut player_disconnected_writer,
                            &mut initial_state_writer,
                            &mut voxel_update_writer,
                            &mut spawn_missile_writer,
                            &mut explode_missile_writer,
                        );
                    },
                    Err(err) => {
                        warn!(?err);
                    }
                }
            },
            Error(error_type) => {
                match error_type {
                    ErrorType::Timeout => {
                        error!("Connection to server timed out!");
                        app_exit_writer.send(AppExit);
                    },
                    ErrorType::Version => {
                        error!("Connection failed: protocol version mismatch!");
                        app_exit_writer.send(AppExit);
                    },
                    ErrorType::Config => {
                        error!("Connection failed: invalid endpoint configuration!");
                        app_exit_writer.send(AppExit);
                    },
                    ErrorType::ServerFull => {
                        error!("Connection failed: server full!");
                        app_exit_writer.send(AppExit);
                    }
                }
            }
        }
    }
}

fn generate_events(
    packet: Packet,
    place_part_command_writer: &mut EventWriter<PlacePartCommand>,
    delete_part_command_writer: &mut EventWriter<DeletePartCommand>,
    player_connected_writer: &mut EventWriter<PlayerConnected>,
    player_disconnected_writer: &mut EventWriter<PlayerDisconnected>,
    initial_state_writer: &mut EventWriter<InitialState>,
    voxel_update_writer: &mut EventWriter<VoxelUpdate>,
    spawn_missile_writer: &mut EventWriter<SpawnMissileCommand>,
    explode_missile_writer: &mut EventWriter<ExplodeMissileCommand>,
) {
    match packet.packet_type() {
        PacketType::PlacePart => {
            match PlacePartCommand::try_from(packet) {
                Ok(place_part_command) => {
                    place_part_command_writer.send(place_part_command);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::DeletePart => {
            match DeletePartCommand::try_from(packet) {
                Ok(delete_part_command) => {
                    delete_part_command_writer.send(delete_part_command);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::PlayerConnected => {
            match PlayerConnected::try_from(packet) {
                Ok(player_connected) => {
                    player_connected_writer.send(player_connected);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::PlayerDisconnected => {
            match PlayerDisconnected::try_from(packet) {
                Ok(player_disconnected) => {
                    player_disconnected_writer.send(player_disconnected);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::InitialState => {
            match InitialState::try_from(packet) {
                Ok(initial_state) => {
                    initial_state_writer.send(initial_state);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::VoxelUpdate => {
            match VoxelUpdate::try_from(packet) {
                Ok(voxel_update) => {
                    voxel_update_writer.send(voxel_update);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::SpawnMissile => {
            match SpawnMissileCommand::try_from(packet) {
                Ok(spawn_missile) => {
                    spawn_missile_writer.send(spawn_missile);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::ExplodeMissile => {
            match ExplodeMissileCommand::try_from(packet) {
                Ok(explode_missile) => {
                    explode_missile_writer.send(explode_missile);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
    }
}