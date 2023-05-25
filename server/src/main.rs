use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use common::PHYSICS_TIMESTEP;
use common::fixed_update::{FixedUpdateSet, SetupFixedTimeStepSchedule};
use common::missile::MissilePlugin;
use common::ship::Ship;
use bevy_rapier3d::prelude::*;

mod missile;
mod network_id_generator;
mod packet_handling;
mod part;
mod player_connection;
mod server_state;

use common::part::{Parts, PartId, PartPlugin};
use common::predefined_parts::add_hardcoded_parts;

use missile::ServerMissilePlugin;
use packet_handling::process_packets;
use part::{ServerPartPlugin, spawn_part};
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;

use crate::player_connection::PlayerConnectionPlugin;

fn main() {
    let rapier_config = RapierConfiguration {
        timestep_mode: TimestepMode::Fixed {
            dt: PHYSICS_TIMESTEP,
            substeps: 1,
        },
        gravity: Vec3::default(),
        ..Default::default()
    };

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
        .setup_fixed_timestep_schedule()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false))
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                .in_base_set(PhysicsSet::SyncBackend)
                .in_schedule(CoreSchedule::FixedUpdate)
        )
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackendFlush)
                .in_base_set(PhysicsSet::SyncBackendFlush)
                .in_schedule(CoreSchedule::FixedUpdate)
        )
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                .in_base_set(PhysicsSet::StepSimulation)
                .in_schedule(CoreSchedule::FixedUpdate)
        )
        .add_systems(
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                .in_base_set(PhysicsSet::Writeback)
                .in_schedule(CoreSchedule::FixedUpdate)
        )
        .add_plugin(PartPlugin)
        .add_plugin(ServerPartPlugin)
        .add_plugin(PlayerConnectionPlugin)
        .add_plugin(MissilePlugin)
        .add_plugin(ServerMissilePlugin)
        .insert_resource(rapier_config)
        .insert_resource(FixedTime::new(Duration::from_secs_f64(1.0 / 60.0)))
        .insert_resource(NetworkIdGenerator::new())
        .add_startup_system(setup_server)
        .add_startup_system(setup)
        .add_system(process_packets.in_schedule(CoreSchedule::FixedUpdate).in_base_set(FixedUpdateSet::PreUpdate))
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