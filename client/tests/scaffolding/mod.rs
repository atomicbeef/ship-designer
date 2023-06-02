use std::time::Duration;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::log::{LogPlugin, Level};
use bevy::diagnostic::DiagnosticsPlugin; 
use bevy::render::{RenderPlugin, settings::WgpuSettings};
use bevy::scene::ScenePlugin;
use bevy::input::{InputPlugin, ButtonState};
use bevy::pbr::PbrPlugin;

use common::fixed_update::{SetupFixedTimeStepSchedule, SetupRapier};
use common::PHYSICS_TIMESTEP;
use ship_designer_client::{app_setup::SetupClientSpecific, connection_state::ConnectionState};
use uflow::client::{Client, Config};

trait SetupClientConnection {
    fn setup_client_connection(&mut self) -> &mut Self;
}

impl SetupClientConnection for App {
    fn setup_client_connection(&mut self) -> &mut Self {
        let server_config = uflow::server::Config {
            max_total_connections: 20,
            max_active_connections: 10,
            enable_handshake_errors: false,
            endpoint_config: uflow::EndpointConfig {
                active_timeout_ms: 3600000,
                ..Default::default()
            }
        };
    
        let mut server = uflow::server::Server::bind_any_ipv4(server_config)
            .expect("Failed to bind socket!");
        
        let mut server_address = server.address();
        server_address.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
        
        let client = Client::connect(server_address, Config::default()).expect("Failed to connect to server!");
        let _ = server.step();
    
        let connection_state = ConnectionState::new(client);
    
        self.insert_resource(connection_state)
    }
}

pub trait SetupBevyPlugins {
    fn setup_bevy_plugins(&mut self) -> &mut Self;
}

impl SetupBevyPlugins for App {
    fn setup_bevy_plugins(&mut self) -> &mut Self {
        self.add_plugins(MinimalPlugins)
            .add_plugin(TransformPlugin)
            .add_plugin(HierarchyPlugin)
            .add_plugin(DiagnosticsPlugin)
            .add_plugin(AssetPlugin::default())
            .add_plugin(ScenePlugin)
            .add_plugin(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    backends: None,
                    ..Default::default()
                }
            })
            .add_plugin(ImagePlugin::default())
            .add_plugin(LogPlugin {
                level: Level::ERROR,
                filter: "wgpu=error,naga=error".to_string(),
            })
            .add_plugin(InputPlugin)
            .add_plugin(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..Default::default()
            })
            .add_plugin(PbrPlugin::default())
    }
}

pub trait ClientTest {
    fn client_test() -> Self;
}

impl ClientTest for App {
    fn client_test() -> Self {
        let mut app = Self::new();

        app.setup_bevy_plugins()
            .setup_fixed_timestep_schedule()
            .setup_rapier()
            .setup_client_specific()
            .setup_client_connection()
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

pub trait MockInput {
    fn mock_key_press(&mut self, key: KeyCode);
    fn mock_key_release(&mut self, key: KeyCode);
    fn mock_mouse_button_press(&mut self, button: MouseButton);
    fn mock_mouse_button_release(&mut self, button: MouseButton);
}

impl MockInput for App {
    fn mock_key_press(&mut self, key: KeyCode) {
        self.world.get_resource_mut::<Events<KeyboardInput>>().unwrap().send(KeyboardInput {
            scan_code: 0,
            key_code: Some(key),
            state: ButtonState::Pressed,
        });
    }

    fn mock_key_release(&mut self, key: KeyCode) {
        self.world.get_resource_mut::<Events<KeyboardInput>>().unwrap().send(KeyboardInput {
            scan_code: 0,
            key_code: Some(key),
            state: ButtonState::Released,
        });
    }

    fn mock_mouse_button_press(&mut self, button: MouseButton) {
        self.world.get_resource_mut::<Events<MouseButtonInput>>().unwrap().send(MouseButtonInput {
            button,
            state: ButtonState::Pressed,
        });
    }

    fn mock_mouse_button_release(&mut self, button: MouseButton) {
        self.world.get_resource_mut::<Events<MouseButtonInput>>().unwrap().send(MouseButtonInput {
            button,
            state: ButtonState::Released,
        });
    }
}

#[test]
fn fixed_update_works() {
    let mut app = App::client_test();

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