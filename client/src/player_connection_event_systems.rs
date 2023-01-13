use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::player::{Player, Players};
use common::shape::{Shapes, ShapeNetworkRepr, ShapeId};
use common::ship::Ship;

use crate::building::spawn_shape;
use crate::meshes::MeshHandles;

pub fn player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut players: ResMut<Players>
) {
    for player_connected in player_connected_reader.iter() {
        info!("{} connected with ID {:?}!", player_connected.name, player_connected.id);
        let player = Player::new(player_connected.id, player_connected.name.clone());
        players.add_player(player)
    }
}

pub fn player_disconnected(
    mut player_disconnected_reader: EventReader<PlayerDisconnected>,
    mut players: ResMut<Players>
) {
    for player_disconnected in player_disconnected_reader.iter() {
        if let Some(player) = players.player(player_disconnected.0) {
            info!("{} disconnected!", player.name());
            players.remove_player(player_disconnected.0);
        }
    }
}

pub fn initial_state_setup(
    mut commands: Commands,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: ResMut<Players>,
    mut initial_state_reader: EventReader<InitialState>,
    mut shapes: ResMut<Shapes>
) {
    for initial_state in initial_state_reader.iter() {
        for player in initial_state.players.iter() {
            players.add_player(player.clone());
        }

        let body = commands.spawn(RigidBody::Dynamic)
            .insert(VisibilityBundle::default())
            .insert(TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)))
            .insert(Velocity::default())
            .insert(GravityScale(0.0))
            .insert(initial_state.body_network_id)
            .insert(Ship)
            .id();

        for (shape_network_repr, transform, network_id) in initial_state.shapes.iter() {
            let shape_handle = match shape_network_repr {
                ShapeNetworkRepr::Predefined(shape_id) => {
                    shapes.get_handle(ShapeId::from(*shape_id))
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
                *network_id,
                body
            );
        }
    }
}