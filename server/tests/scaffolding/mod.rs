use std::time::Duration;

use bevy::{prelude::*, log::{LogPlugin, Level}};

use common::{fixed_update::{SetupFixedTimeStepSchedule, SetupRapier}, PHYSICS_TIMESTEP};
use ship_designer_server::{app_setup::{SetupBevyPlugins, SetupServerSpecific}, server_state::ServerState};

fn setup_server(world: &mut World) {
    let server_config = uflow::server::Config {
        max_total_connections: 20,
        max_active_connections: 10,
        enable_handshake_errors: false,
        endpoint_config: uflow::EndpointConfig {
            active_timeout_ms: 3600000,
            ..Default::default()
        }
    };

    let server = uflow::server::Server::bind_any_ipv4(server_config)
        .expect("Failed to bind socket!");
    
    let server_state = ServerState::new(server);

    world.insert_non_send_resource(server_state);
}

pub trait ServerTest {
    fn server_test() -> Self;
}

impl ServerTest for App {
    fn server_test() -> Self {
        let mut app = Self::new();

        app.setup_bevy_plugins()
            .add_plugin(LogPlugin {
                level: Level::ERROR,
                filter: String::new(),
            })
            .setup_fixed_timestep_schedule()
            .setup_rapier()
            .setup_server_specific()
            .add_startup_system(setup_server)
            .setup();

        app
    }
}

pub trait FixedUpdate {
    fn fixed_update(&mut self);
}

impl FixedUpdate for App {
    fn fixed_update(&mut self) {
        let mut time = self.world.get_resource_mut::<FixedTime>().unwrap();

        let accumulated = time.accumulated();
        if accumulated < Duration::from_secs_f32(PHYSICS_TIMESTEP) {
            time.tick(Duration::from_secs_f32(PHYSICS_TIMESTEP) - accumulated);
        }

        self.update();
    }
}

#[test]
fn fixed_update_works() {
    let mut app = App::server_test();

    #[derive(Resource)]
    struct TestResource(pub u32);

    fn test_me(mut test_resource: ResMut<TestResource>) {
        test_resource.0 += 1;
    }

    app.insert_resource(TestResource(0));
    app.add_system(test_me.in_schedule(CoreSchedule::FixedUpdate));

    app.fixed_update();

    assert_eq!(1, app.world.get_resource::<TestResource>().unwrap().0);
}