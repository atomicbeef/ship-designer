use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use common::index::IndexPlugin;
use common::network_id::NetworkId;
use common::player::Players;
use common::ship::Ship;
use bevy_rapier3d::prelude::*;

mod network_id_generator;
mod packet_handling;
mod part;
mod player_connection;
mod server_state;

use common::part::{Parts, PartId, PartPlugin};
use common::predefined_parts::add_hardcoded_parts;

use packet_handling::process_packets;
use part::{ServerPartPlugin, spawn_part};
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;

use crate::player_connection::PlayerConnectionPlugin;

fn main() {
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
        .add_plugin(MeshPlugin)
        .add_plugin(ScenePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(IndexPlugin::<NetworkId>::new())
        .add_plugin(PartPlugin)
        .add_plugin(ServerPartPlugin)
        .add_plugin(PlayerConnectionPlugin)
        .insert_resource(FixedTime::new(Duration::from_millis(16)))
        .insert_resource(NetworkIdGenerator::new())
        .insert_resource(Players::new())
        .add_startup_system(setup_server)
        .add_startup_system(setup)
        .add_system(process_packets.in_schedule(CoreSchedule::FixedUpdate))
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

    let construct = commands.spawn(RigidBody::Dynamic)
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
        construct
    );
}