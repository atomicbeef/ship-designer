use app_setup::{setup_hardcoded_parts, SetupBevyPlugins, SetupRapier, SetupServerSpecific};
use bevy::prelude::*;
use common::fixed_update::{FixedUpdateSet, SetupFixedTimeStepSchedule};
use common::ship::Ship;
use bevy_rapier3d::prelude::*;

mod app_setup;
mod missile;
mod network_id_generator;
mod packet_handling;
mod part;
mod player_connection;
mod server_state;

use common::part::{Parts, PartId};

use packet_handling::process_packets;
use part::spawn_part;
use server_state::ServerState;
use network_id_generator::NetworkIdGenerator;

fn main() {
    App::new()
        .setup_bevy_plugins()
        .setup_fixed_timestep_schedule()
        .setup_rapier()
        .setup_server_specific()
        .add_startup_system(setup_server.after(setup_hardcoded_parts))
        .add_startup_system(setup.after(setup_hardcoded_parts))
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