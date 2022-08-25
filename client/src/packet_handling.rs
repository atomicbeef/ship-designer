use bevy::prelude::*;
use uflow::Event::*;

use common::events::{PlaceBlockCommand, DeleteBlockCommand};
use common::packets::{Packet, PacketType};

use crate::connection_state::ConnectionState;

pub fn process_packets(
    mut state: ResMut<ConnectionState>,
    mut place_block_command_writer: EventWriter<PlaceBlockCommand>,
    mut delete_block_command_writer: EventWriter<DeleteBlockCommand>
) {
    state.client.step();

    for event in state.server.poll_events() {
        match event {
            Connect => {
                info!("Connected to server");
            },
            Disconnect => {
                info!("Disconnected from server");
            },
            Timeout => {
                info!("Connection to server timed out!");
            },
            Receive(packet_data) => {
                match Packet::try_from(packet_data) {
                    Ok(packet) => {
                        generate_events(packet, &mut place_block_command_writer, &mut delete_block_command_writer);
                    },
                    Err(err) => {
                        warn!(?err);
                    }
                }
            }
        }
    }
}

fn generate_events(
    packet: Packet,
    place_black_command_writer: &mut EventWriter<PlaceBlockCommand>,
    delete_block_command_writer: &mut EventWriter<DeleteBlockCommand>
) {
    match packet.packet_type() {
        PacketType::PlaceBlock => {
            match PlaceBlockCommand::try_from(packet) {
                Ok(place_block_command) => {
                    place_black_command_writer.send(place_block_command);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        },
        PacketType::DeleteBlock => {
            match DeleteBlockCommand::try_from(packet) {
                Ok(delete_block_command) => {
                    delete_block_command_writer.send(delete_block_command);
                },
                Err(err) => {
                    warn!(?err);
                }
            }
        }
    }
}