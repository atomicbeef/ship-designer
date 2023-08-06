use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{WindowClosed, PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use uflow::client::Client;
use uflow::EndpointConfig;

use common::part::{Parts, PartId};
use common::fixed_update::{FixedUpdateSet, SetupFixedTimeStepSchedule, SetupRapier};

use ship_designer_client::app_setup::{SetupClientSpecific, setup_hardcoded_parts};
use ship_designer_client::fixed_input::FixedInputSystem;
use ship_designer_client::raycast_selection::{update_intersections, SelectionSource};
use ship_designer_client::building::BuildMarkerBundle;
use ship_designer_client::free_camera::FreeCamera;
use ship_designer_client::connection_state::ConnectionState;
use ship_designer_client::part::meshes::PartMeshHandles;

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
        .add_plugins(RapierDebugRenderPlugin::default())
        .setup_client_specific()
        .add_systems(Startup, (
            set_window_title,
            setup.after(setup_hardcoded_parts)
        ))
        .add_systems(Update, disconnect_on_esc)
        .add_systems(Update, disconnect_on_window_close)
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(connection_state)
        .add_systems(FixedUpdate,
            update_intersections.in_set(FixedUpdateSet::PreUpdate).after(FixedInputSystem)
        )
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

    commands.spawn(BuildMarkerBundle::new(
        PartId::from(1),
        &parts,
        &mut mesh_handles,
        &mut meshes,
        &mut materials,
    ));
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