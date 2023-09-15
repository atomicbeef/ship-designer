use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_rapier3d::prelude::*;

use common::entity_lookup::lookup;
use common::fixed_update::{AddFixedEvent, FixedUpdateSet};
use common::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::player::{PlayerId, PlayerName, PlayerBundle};
use common::part::{Parts, PartNetworkRepr, PartId};
use common::ship::Ship;

use crate::camera::ActiveCameraEntity;
use crate::part::spawn_part;
use crate::building_material::BuildingMaterial;
use crate::part::meshes::PartMeshHandles;
use crate::player_controller::{ControlledPlayer, PlayerCamera};
use crate::raycast_selection::SelectionSource;

fn player_connected(
    mut player_connected_reader: EventReader<PlayerConnected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for player_connected in player_connected_reader.iter() {
        info!("{} connected with ID {:?}!", player_connected.name, player_connected.id);

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.5,
                    depth: 2.0,
                    ..Default::default()
                })),
            material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
            ..Default::default()
        }).insert(PlayerBundle {
            id: player_connected.id,
            name: player_connected.name.clone(),
            transform: TransformBundle::from_transform(player_connected.transform),
            ..Default::default()
        });
    }
}

fn player_disconnected(
    mut player_disconnected_reader: EventReader<PlayerDisconnected>,
    mut commands: Commands,
    player_entity_query: Query<(Entity, &PlayerId)>,
    name_query: Query<&PlayerName>
) {
    for player_disconnected in player_disconnected_reader.iter() {
        if let Some(entity) = lookup(&player_entity_query, &player_disconnected.0) {
            let name = name_query.get(entity).unwrap();
            info!("{} disconnected!", name);
            commands.entity(entity).despawn();
        }
    }
}

fn initial_state_setup(
    mut commands: Commands,
    mut mesh_handles: ResMut<PartMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut building_materials: ResMut<Assets<BuildingMaterial>>,
    mut initial_state_reader: EventReader<InitialState>,
    mut parts: ResMut<Parts>,
    mut active_camera: ResMut<ActiveCameraEntity>,
) {
    for initial_state in initial_state_reader.iter() {
        for (id, name, transform) in initial_state.players.iter() {
            let player = commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                        radius: 0.5,
                        depth: 2.0,
                        ..Default::default()
                    })),
                material: standard_materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                ..Default::default()
            }).insert(PlayerBundle {
                id: *id,
                name: name.clone(),
                transform: TransformBundle::from_transform(*transform),
                ..Default::default()
            }).id();

            if *id == initial_state.player_id {
                commands.entity(player)
                    .insert(ControlledPlayer)
                    // Make the controller player invisible to the first person camera
                    .insert(RenderLayers::from_layers(&[1]))
                    .with_children(|parent| {
                        let id = parent.spawn(Camera3dBundle {
                            transform: Transform::from_xyz(0.0, 1.95, 0.0),
                            ..Default::default()
                        })
                            .insert(SelectionSource::new())
                            .insert(PlayerCamera)
                            .id();
                        active_camera.0 = Some(id);
                    });
            }
        }

        let construct = commands.spawn(RigidBody::Dynamic)
            .insert(VisibilityBundle::default())
            .insert(TransformBundle::from_transform(Transform::from(initial_state.construct_transform)))
            .insert(Velocity::default())
            .insert(initial_state.construct_network_id)
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
                &mut building_materials,
                &parts,
                part_handle,
                Transform::from(*transform),
                *network_id,
                construct
            );
        }
    }
}

pub struct PlayerConnectionPlugin;

impl Plugin for PlayerConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_fixed_event::<PlayerConnected>()
            .add_fixed_event::<PlayerDisconnected>()
            .add_fixed_event::<InitialState>()
            .add_systems(FixedUpdate, (
                player_connected,
                player_disconnected,
                initial_state_setup.run_if(on_event::<InitialState>()),
            ).in_set(FixedUpdateSet::Update));
    }
}