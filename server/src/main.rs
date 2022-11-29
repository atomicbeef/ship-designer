use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

mod building_systems;
mod network_id_generator;
mod packet_handling;
mod player_connection_event_systems;
mod server_state;

use building_systems::{send_place_shape_commands, send_delete_shape_commands};
use common::events::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand, PlayerConnected, PlayerDisconnected};
use common::shape::{Shapes, ShapeHandle, ShapeHandleId, ShapeHandleType};
use common::predefined_shapes::add_hardcoded_shapes;

use building_systems::{confirm_place_shape_requests, confirm_delete_shape_requests};
use packet_handling::process_packets;
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;
use player_connection_event_systems::{send_player_connected, send_player_disconnected};

#[derive(StageLabel)]
struct NetworkStage;

fn main() {
    let address = "127.0.0.1:36756";
    let max_peer_count = 12;
    let peer_config = uflow::EndpointConfig::default();
    let server = uflow::Server::bind(address, max_peer_count, peer_config)
        .expect(&format!("Failed to bind on {}", address));
    
    let server_state = ServerState::new(server);

    let mut packet_process_stage = SystemStage::parallel();
    packet_process_stage.add_system(process_packets);

    let mut shapes = Shapes::new();
    add_hardcoded_shapes(&mut shapes);

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin {
            level: Level::DEBUG,
            filter: String::new()
        })
        .add_plugin(TransformPlugin)
        .add_plugin(HierarchyPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(AssetPlugin::default())
        .insert_resource(server_state)
        .insert_resource(NetworkIdGenerator::new())
        .insert_resource(shapes)
        .add_event::<PlaceShapeRequest>()
        .add_event::<PlaceShapeCommand>()
        .add_event::<DeleteShapeRequest>()
        .add_event::<DeleteShapeCommand>()
        .add_event::<PlayerConnected>()
        .add_event::<PlayerDisconnected>()
        .add_stage_before(
            CoreStage::Update,
            NetworkStage,
            FixedTimestepStage::new(Duration::from_millis(16), "network_stage")
                .with_stage(packet_process_stage)
        )
        .add_startup_system(setup)
        .add_system(confirm_place_shape_requests)
        .add_system(confirm_delete_shape_requests)
        .add_system(send_place_shape_commands)
        .add_system(send_delete_shape_commands)
        .add_system(send_player_connected)
        .add_system(send_player_disconnected)
        .run();
}

fn setup(mut commands: Commands, mut network_id_generator: ResMut<NetworkIdGenerator>) {
    let shape_handle = ShapeHandle::new(ShapeHandleId::from(0), ShapeHandleType::ReadOnly);
    let network_id = network_id_generator.generate();
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn_empty()
        .insert(shape_handle)
        .insert(network_id)
        .insert(transform);
}