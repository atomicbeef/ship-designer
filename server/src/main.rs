use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use common::colliders::{remove_unused_colliders, RegenerateColliders};
use common::index::{update_index, Index};
use common::network_id::NetworkId;
use common::player::Players;
use common::ship::Ship;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;

mod building_systems;
mod network_id_generator;
mod packet_handling;
mod player_connection_event_systems;
mod server_state;

use building_systems::{send_place_part_commands, send_delete_part_commands, spawn_part, regenerate_colliders};
use common::events::building::{PlacePartRequest, PlacePartCommand, DeletePartRequest, DeletePartCommand};
use common::events::player_connection::{PlayerConnected, PlayerDisconnected};
use common::part::{Parts, PartId, free_parts, FreedParts};
use common::predefined_parts::add_hardcoded_parts;

use building_systems::{confirm_place_part_requests, confirm_delete_part_requests};
use packet_handling::process_packets;
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;
use player_connection_event_systems::{send_player_connected, send_player_disconnected};

#[derive(StageLabel)]
struct NetworkStage;

fn main() {
    let mut packet_process_stage = SystemStage::parallel();
    packet_process_stage.add_system(process_packets);

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
        .insert_resource(Parts::new())
        .insert_resource(Players::new())
        .insert_resource(Index::<NetworkId>::new())
        .add_event::<RegenerateColliders>()
        .add_event::<FreedParts>()
        .add_event::<PlacePartRequest>()
        .add_event::<PlacePartCommand>()
        .add_event::<DeletePartRequest>()
        .add_event::<DeletePartCommand>()
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
        .add_system(free_parts)
        .add_system(confirm_place_part_requests)
        .add_system(confirm_delete_part_requests)
        .add_system(send_place_part_commands)
        .add_system(send_delete_part_commands)
        .add_system(send_player_connected)
        .add_system(send_player_disconnected)
        .add_system(remove_unused_colliders)
        .add_system(regenerate_colliders)
        .add_system_to_stage(CoreStage::PostUpdate, update_index::<NetworkId>)
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

fn setup(
    mut commands: Commands,
    mut network_id_generator: ResMut<NetworkIdGenerator>,
    mut parts: ResMut<Parts>
) {
    add_hardcoded_parts(&mut parts);

    let body = commands.spawn(RigidBody::Dynamic)
        .insert(VisibilityBundle::default())
        .insert(TransformBundle::from_transform(Transform {
            //translation: Vec3::new(1.0, -1.0, 1.0),
            translation: Vec3::splat(0.0),
            rotation: Quat::IDENTITY,
            //rotation: Quat::from_xyzw(0.002, 0.612, -0.204, -0.764).normalize(),
            scale: Vec3::splat(1.0)
        }))
        .insert(Velocity::default())
        .insert(GravityScale(0.0))
        .insert(network_id_generator.generate())
        .insert(Ship)
        .id();
    
    let part_handle = parts.get_handle(PartId::from(0));

    spawn_part(
        &mut commands,
        &mut parts,
        part_handle,
        Transform::from_xyz(0.0, 0.0, 0.0),
        network_id_generator.generate(),
        body
    );
}