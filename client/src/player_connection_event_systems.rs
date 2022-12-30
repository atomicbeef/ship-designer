use bevy::prelude::*;

use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::player::Player;
use common::shape::{Shapes, ShapeNetworkRepr, ShapeHandle};

use crate::building::spawn_shape;
use crate::connection_state::ConnectionState;
use crate::meshes::MeshHandles;

pub fn player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut connection_state: ResMut<ConnectionState>
) {
    for player_connected in player_connected_reader.iter() {
        info!("{} connected with ID {}!", player_connected.name, player_connected.id);
        let player = Player::new(player_connected.id, player_connected.name.clone());
        connection_state.add_player(player)
    }
}

pub fn player_disconnected(
    mut player_disconnected_reader: EventReader<PlayerDisconnected>,
    mut connection_state: ResMut<ConnectionState>
) {
    for player_disconnected in player_disconnected_reader.iter() {
        if let Some(player) = connection_state.player(player_disconnected.0) {
            info!("{} disconnected!", player.name());
            connection_state.remove_player(player_disconnected.0);
        }
    }
}

pub fn initial_state_setup(
    mut commands: Commands,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut connection_state: ResMut<ConnectionState>,
    mut initial_state_reader: EventReader<InitialState>,
    mut shapes: ResMut<Shapes>
) {
    for initial_state in initial_state_reader.iter() {
        for player in initial_state.players.iter() {
            connection_state.add_player(player.clone());
        }

        for (shape_network_repr, transform, network_id) in initial_state.shapes.iter() {
            let shape_handle = match shape_network_repr {
                ShapeNetworkRepr::Predefined(shape_id) => {
                    ShapeHandle::new(*shape_id)
                },
                ShapeNetworkRepr::Child(shape) => {
                    shapes.add(shape.clone())
                }
            };

            spawn_shape(
                &mut commands,
                &mut mesh_handles,
                &mut meshes,
                &mut materials,
                &shapes,
                shape_handle,
                Transform::from(*transform),
                *network_id
            );
        }
    }
}