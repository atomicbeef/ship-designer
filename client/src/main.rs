use std::time::Duration;

use bevy::log::{Level, LogSettings};
use bevy::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, DebugCursorPickingPlugin, PickingCameraBundle};
use iyes_loopless::prelude::*;
use uflow::Client;

mod building;
mod camera;
mod connection_state;
mod packet_handling;
mod player_connection_event_systems;
mod settings;

use common::grid::Grid;
use common::events::{PlaceBlockRequest, PlaceBlockCommand, DeleteBlockRequest, DeleteBlockCommand, PlayerConnected, PlayerDisconnected, InitialState};
use crate::building::{build_request_events, place_blocks, delete_blocks, send_place_block_requests, send_delete_block_requests};
use crate::camera::{FreeCameraPlugin, FreeCamera};
use crate::connection_state::ConnectionState;
use crate::packet_handling::process_packets;
use crate::player_connection_event_systems::{player_connected, player_disconnected, initial_state_setup};

#[derive(StageLabel)]
struct NetworkStage;

fn main() {
    let mut client = Client::bind_any_ipv4().expect("Failed to bind socket!");
    let server_address = "127.0.0.1:36756";
    let peer_config = uflow::EndpointConfig::default();

    let server = client.connect(server_address, peer_config).expect("Failed to connect to server!");

    let connection_state = ConnectionState::new(client, server);

    let mut packet_process_stage = SystemStage::parallel();
    packet_process_stage.add_system(process_packets);

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(settings::Settings::default())
        .insert_resource(LogSettings {
            level: Level::WARN,
            filter: "wgpu=error".to_string()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FreeCameraPlugin)
        .add_startup_system(set_window_title)
        .add_startup_system(setup)
        .add_system(disconnect_on_esc)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .insert_resource(connection_state)
        .add_stage_before(
            CoreStage::Update,
            NetworkStage,
            FixedTimestepStage::new(Duration::from_millis(16))
                .with_stage(packet_process_stage)
        )
        .add_event::<PlaceBlockRequest>()
        .add_event::<PlaceBlockCommand>()
        .add_event::<DeleteBlockRequest>()
        .add_event::<DeleteBlockCommand>()
        .add_event::<PlayerConnected>()
        .add_event::<PlayerDisconnected>()
        .add_event::<InitialState>()
        .add_system(build_request_events)
        .add_system(send_place_block_requests)
        .add_system(send_delete_block_requests)
        .add_system(place_blocks)
        .add_system(delete_blocks)
        .add_system(player_connected)
        .add_system(player_disconnected)
        .add_system(initial_state_setup)
        .run();
}

fn set_window_title(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_title("Ship Designer".to_string());
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert_bundle(PickingCameraBundle::default());

    commands.spawn().insert(Grid::new());
}

fn disconnect_on_esc(
    keys: Res<Input<KeyCode>>,
    connection_state: ResMut<ConnectionState>
) {
    if keys.pressed(KeyCode::Escape) {
        connection_state.server.disconnect();
    }
}