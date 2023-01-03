use std::time::Duration;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::WindowClosed;
use bevy_mod_picking::{DefaultPickingPlugins, DebugCursorPickingPlugin, PickingCameraBundle};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use common::player::Players;
use iyes_loopless::prelude::*;
use mesh_generation::{RegenerateShapeMesh, regenerate_shape_mesh};
use uflow::client::Client;
use uflow::EndpointConfig;

use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::events::player_connection::{PlayerConnected, PlayerDisconnected, InitialState};
use common::predefined_shapes::add_hardcoded_shapes;
use common::shape::{Shapes, ShapeHandle};

mod building;
mod camera;
mod connection_state;
mod meshes;
mod mesh_generation;
mod packet_handling;
mod player_connection_event_systems;
mod settings;

use building::{build_request_events, place_shapes, delete_shapes, send_place_block_requests, send_delete_block_requests};
use camera::{FreeCameraPlugin, FreeCamera};
use connection_state::ConnectionState;
use meshes::MeshHandles;
use packet_handling::process_packets;
use player_connection_event_systems::{player_connected, player_disconnected, initial_state_setup};

#[derive(StageLabel)]
struct NetworkStage;

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

    let mut packet_process_stage = SystemStage::parallel();
    packet_process_stage.add_system(process_packets);

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(settings::Settings::default())
        .add_plugins(
            DefaultPlugins.set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,naga=error".to_string()
            })
            .set(WindowPlugin {
                exit_on_all_closed: false,
                ..default()
            })
        )
        .add_plugin(FreeCameraPlugin)
        .add_startup_system(set_window_title)
        .add_startup_system(setup)
        .add_system(disconnect_on_esc)
        .add_system(disconnect_on_window_close)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(connection_state)
        .insert_resource(Players::new())
        .insert_resource(Shapes::new())
        .insert_resource(MeshHandles::new())
        .add_stage_before(
            CoreStage::Update,
            NetworkStage,
            FixedTimestepStage::new(Duration::from_millis(16), "network_stage")
                .with_stage(packet_process_stage)
        )
        .add_event::<PlaceShapeRequest>()
        .add_event::<PlaceShapeCommand>()
        .add_event::<DeleteShapeRequest>()
        .add_event::<DeleteShapeCommand>()
        .add_event::<PlayerConnected>()
        .add_event::<PlayerDisconnected>()
        .add_event::<InitialState>()
        .add_event::<RegenerateShapeMesh>()
        .add_system(build_request_events)
        .add_system(send_place_block_requests)
        .add_system(send_delete_block_requests)
        .add_system(place_shapes)
        .add_system(delete_shapes)
        .add_system(regenerate_shape_mesh)
        .add_system(player_connected)
        .add_system(player_disconnected)
        .add_system(initial_state_setup.run_on_event::<InitialState>())
        .register_type::<ShapeHandle>()
        .run();
}

fn set_window_title(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_title("Ship Designer".to_string());
}

fn setup(
    mut shapes: ResMut<Shapes>,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands
) {
    let shape_handles = add_hardcoded_shapes(&mut shapes);

    for shape_handle in shape_handles {
        let shape = shapes.get(&shape_handle).unwrap();
        let mesh = mesh_generation::generate_shape_mesh(shape);
        let mesh_handle = meshes.add(mesh);
        mesh_handles.add(shape_handle, mesh_handle);
    }

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert(PickingCameraBundle::default());
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