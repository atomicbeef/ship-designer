use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::player::{Player, Players};
use common::part::{Parts, PartNetworkRepr, PartId};
use common::ship::Ship;

use crate::building::spawn_part;
use crate::building_material::BuildingMaterial;
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
    mut materials: ResMut<Assets<BuildingMaterial>>,
    mut players: ResMut<Players>,
    mut initial_state_reader: EventReader<InitialState>,
    mut parts: ResMut<Parts>
) {
    for initial_state in initial_state_reader.iter() {
        for player in initial_state.players.iter() {
            players.add_player(player.clone());
        }

        let body = commands.spawn(RigidBody::Dynamic)
            .insert(VisibilityBundle::default())
            .insert(TransformBundle::from_transform(Transform::from(initial_state.body_transform)))
            .insert(Velocity::default())
            .insert(GravityScale(0.0))
            .insert(initial_state.body_network_id)
            .insert(Ship)
            .id();

        for (part_network_repr, transform, network_id) in initial_state.parts.iter() {
            let part_handle = match part_network_repr {
                PartNetworkRepr::Predefined(part_id) => {
                    parts.get_handle(PartId::from(*part_id))
                },
                PartNetworkRepr::Child(part) => {
                    parts.add(part.clone())
                }
            };

            spawn_part(
                &mut commands,
                &mut mesh_handles,
                &mut meshes,
                &mut materials,
                &parts,
                part_handle,
                Transform::from(*transform),
                *network_id,
                body
            );
        }
    }
}