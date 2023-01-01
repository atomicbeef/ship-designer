use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use common::player::Players;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;

mod building_systems;
mod network_id_generator;
mod packet_handling;
mod player_connection_event_systems;
mod server_state;

use building_systems::{send_place_shape_commands, send_delete_shape_commands};
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::events::player_connection::{PlayerConnected, PlayerDisconnected};
use common::shape::{Shapes, ShapeHandle, ShapeId};
use common::predefined_shapes::add_hardcoded_shapes;

use building_systems::{confirm_place_shape_requests, confirm_delete_shape_requests};
use packet_handling::process_packets;
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;
use player_connection_event_systems::{send_player_connected, send_player_disconnected};

#[derive(StageLabel)]
struct NetworkStage;

fn main() {
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
        .add_plugin(ScenePlugin)
        .add_plugin(MeshPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(NetworkIdGenerator::new())
        .insert_resource(shapes)
        .insert_resource(Players::new())
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
        .add_startup_system(setup_server)
        .add_startup_system(setup)
        .add_system(confirm_place_shape_requests)
        .add_system(confirm_delete_shape_requests)
        .add_system(send_place_shape_commands)
        .add_system(send_delete_shape_commands)
        .add_system(send_player_connected)
        .add_system(send_player_disconnected)
        .run();
}

fn setup_server(world: &mut World) {
    let address = "127.0.0.1:36756";
    let server_config = uflow::server::Config {
        max_total_connections: 20,
        max_active_connections: 10,
        enable_handshake_errors: false,
        endpoint_config: uflow::EndpointConfig {
            active_timeout_ms: 3600000,
            ..Default::default()
        }
    };

    let server = uflow::server::Server::bind(address, server_config)
        .expect(&format!("Failed to bind on {}", address));
    
    let server_state = ServerState::new(server);

    world.insert_non_send_resource(server_state);
}

fn setup(mut commands: Commands, mut network_id_generator: ResMut<NetworkIdGenerator>) {
    let shape_handle = ShapeHandle::new(ShapeId::from(0));
    let network_id = network_id_generator.generate();
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn_empty()
        .insert(shape_handle)
        .insert(network_id)
        .insert(transform);
}