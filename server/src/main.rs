use bevy::log::{LogPlugin, Level};
use bevy::prelude::*;

use common::part::{Parts, PartId};
use common::fixed_update::{SetupFixedTimeStepSchedule, SetupRapier};
use common::ship::ShipBundle;
use ship_designer_server::app_setup::{setup_hardcoded_parts, SetupBevyPlugins, SetupServerSpecific};
use ship_designer_server::part::spawn_part;
use ship_designer_server::server_state::ServerState;
use ship_designer_server::network_id_generator::NetworkIdGenerator;

fn main() {
    App::new()
        .setup_bevy_plugins()
        .add_plugin(LogPlugin {
            level: Level::DEBUG,
            filter: String::new()
        })
        .setup_fixed_timestep_schedule()
        .setup_rapier()
        .setup_server_specific()
        .add_startup_system(setup_server.after(setup_hardcoded_parts))
        .add_startup_system(setup.after(setup_hardcoded_parts))
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
    let construct = commands.spawn(ShipBundle {
        transform: TransformBundle::from_transform(Transform {
            //translation: Vec3::new(1.0, -1.0, 1.0),
            translation: Vec3::splat(0.0),
            rotation: Quat::IDENTITY,
            //rotation: Quat::from_xyzw(0.002, 0.612, -0.204, -0.764).normalize(),
            scale: Vec3::splat(1.0)
        }),
        network_id: network_id_generator.generate(),
        ..Default::default()
    }).id();
    
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