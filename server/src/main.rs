use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{LogSettings, Level};
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use iyes_loopless::prelude::*;

mod building_systems;
mod packet_handling;
mod server_state;

use building_systems::{send_place_block_commands, send_delete_block_commands};
use common::grid::Grid;
use common::events::{PlaceBlockRequest, PlaceBlockCommand, DeleteBlockRequest, DeleteBlockCommand};

use crate::building_systems::{confirm_place_block_requests, confirm_delete_block_requests};
use crate::packet_handling::process_packets;
use crate::server_state::ServerState;

#[derive(StageLabel)]
struct NetworkStage;

fn main() {
    let address = "127.0.0.1:36756";
    let max_peer_count = 2;
    let peer_config = uflow::EndpointConfig::default();
    let server = uflow::Server::bind(address, max_peer_count, peer_config)
        .expect(&format!("Failed to bind on {}", address));
    
    let server_state = ServerState { server, peer_list: Vec::new() };

    let mut packet_process_stage = SystemStage::parallel();
    packet_process_stage.add_system(process_packets);

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            filter: String::new()
        })
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(HierarchyPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(ScenePlugin)
        .add_startup_system(setup)
        .insert_resource(server_state)
        .add_event::<PlaceBlockRequest>()
        .add_event::<PlaceBlockCommand>()
        .add_event::<DeleteBlockRequest>()
        .add_event::<DeleteBlockCommand>()
        .add_stage_before(
            CoreStage::Update,
            NetworkStage,
            FixedTimestepStage::new(Duration::from_millis(16))
                .with_stage(packet_process_stage)
        )
        .add_system(confirm_place_block_requests)
        .add_system(confirm_delete_block_requests)
        .add_system(send_place_block_commands)
        .add_system(send_delete_block_commands)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn().insert(Grid::new());
}
