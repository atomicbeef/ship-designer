use app_setup::{SetupClientSpecific, setup_hardcoded_parts};
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{WindowClosed, PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use common::fixed_update::{FixedUpdateSet, SetupFixedTimeStepSchedule, SetupRapier};
use fixed_input::FixedInputSystem;
use raycast_selection::{update_intersections, SelectionSource};
use uflow::client::Client;
use uflow::EndpointConfig;

use common::part::{Parts, PartId};

mod app_setup;
mod building;
mod building_material;
mod camera;
mod connection_state;
mod fixed_input;
mod missile;
mod part;
mod packet_handling;
mod player_connection;
mod player_controller;
mod raycast_selection;
mod settings;

use building::{BuildMarker, BuildMarkerOrientation};
use camera::FreeCamera;
use connection_state::ConnectionState;
use part::meshes::PartMeshHandles;

fn main() {
    let server_address = "127.0.0.1:36756";
    let client_config = uflow::client::Config {
        endpoint_config: EndpointConfig {
            active_timeout_ms: 3600000,
            ..Default::default()
        }
    };

    let client = Client::connect(server_address, client_config).expect("Failed to connect to server!");

    let connection_state = ConnectionState::new(client);
    
    App::new().insert_resource(Msaa::default())
        .add_plugins(
            DefaultPlugins.set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,naga=error".to_string()
            })
            .set(WindowPlugin {
                exit_condition: bevy::window::ExitCondition::OnAllClosed,
                ..default()
            })
        )
        .setup_fixed_timestep_schedule()
        .setup_rapier()
        .add_plugin(RapierDebugRenderPlugin::default())
        .setup_client_specific()
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(set_window_title)
        .add_startup_system(setup.after(setup_hardcoded_parts))
        .add_system(disconnect_on_esc)
        .add_system(disconnect_on_window_close)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(connection_state)
        .add_system(update_intersections.in_base_set(FixedUpdateSet::PreUpdate).after(FixedInputSystem))
        .register_type::<common::part::PartId>()
        .register_type::<common::part::PartHandle>()
        .register_type::<common::player::PlayerId>()
        .register_type::<common::player::PlayerName>()
        .run();
}

fn set_window_title(mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,) {
    let mut window = primary_window_query.single_mut();

    window.title = "Ship Designer".to_string();
}

fn setup(
    parts: Res<Parts>,
    mut mesh_handles: ResMut<PartMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert(SelectionSource::new());

    let marker_part_handle = parts.get_handle(PartId::from(1));
    let marker_part = parts.get(&marker_part_handle).unwrap();
    // If we use exactly the part bounds, then we can't place parts next to each other
    let marker_half_extents = marker_part.center() - Vec3::splat(0.01);

    commands.spawn(BuildMarker)
        .insert(BuildMarkerOrientation(Quat::IDENTITY))
        .insert(PbrBundle {
            mesh: part::meshes::get_mesh_or_generate(marker_part_handle.id(), marker_part, &mut mesh_handles, &mut meshes),
            material: materials.add(Color::rgba(0.25, 0.62, 0.26, 0.5).into()),
            ..Default::default()
        })
        .insert(marker_part_handle)
        .insert(Collider::cuboid(
            marker_half_extents.x,
            marker_half_extents.y,
            marker_half_extents.z
        ))
        .insert(Sensor);
}

fn disconnect_on_esc(
    keys: Res<Input<KeyCode>>,
    mut connection_state: ResMut<ConnectionState>
) {
    if keys.pressed(KeyCode::Escape) {
        connection_state.client.disconnect();
    }
}

fn disconnect_on_window_close(
    window_closed: EventReader<WindowClosed>,
    mut connection_state: ResMut<ConnectionState>
) {
    if !window_closed.is_empty() {
        connection_state.client.disconnect();
    }
}