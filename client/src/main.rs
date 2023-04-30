use std::time::Duration;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{WindowClosed, PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::*;
use common::missile::MissilePlugin;
use missile::ClientMissilePlugin;
use raycast_selection::{update_intersections, SelectionSource};
use uflow::client::Client;
use uflow::EndpointConfig;

use common::player::Players;
use common::predefined_parts::add_hardcoded_parts;
use common::part::{Parts, PartId, PartPlugin};

mod building;
mod building_material;
mod camera;
mod connection_state;
mod missile;
mod part;
mod packet_handling;
mod player_connection;
mod raycast_selection;
mod settings;

use building::{BuildMarker, BuildMarkerOrientation, BuildingPlugin};
use camera::{FreeCameraPlugin, FreeCamera};
use connection_state::ConnectionState;
use part::meshes::PartMeshHandles;
use packet_handling::process_packets;
use part::ClientPartPlugin;
use player_connection::PlayerConnectionPlugin;

fn main() {
    let server_address = "127.0.0.1:36756";
    let client_config = uflow::client::Config {
        endpoint_config: EndpointConfig {
            active_timeout_ms: 3600000,
            ..Default::default()
        }
    };

    let client = Client::connect(server_address, client_config).expect("Failed to connect to server!");

    let connection_state = ConnectionState::new(client);

    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(settings::Settings::default())
        .add_plugins(
            DefaultPlugins.set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,naga=error".to_string()
            })
            .set(WindowPlugin {
                exit_condition: bevy::window::ExitCondition::OnAllClosed,
                ..default()
            })
        )
        .add_plugin(FreeCameraPlugin)
        .add_plugin(BuildingPlugin)
        .add_plugin(PlayerConnectionPlugin)
        .add_plugin(PartPlugin)
        .add_plugin(ClientPartPlugin)
        .add_plugin(MissilePlugin)
        .add_plugin(ClientMissilePlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system(set_window_title)
        .add_startup_system(setup)
        .add_system(disconnect_on_esc)
        .add_system(disconnect_on_window_close)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(FixedTime::new(Duration::from_millis(16)))
        .insert_resource(connection_state)
        .insert_resource(Players::new())
        .add_system(process_packets.in_schedule(CoreSchedule::FixedUpdate))
        .add_system(update_intersections.in_base_set(CoreSet::First))
        .register_type::<common::part::PartId>()
        .register_type::<common::part::PartHandle>()
        .run();
}

fn set_window_title(mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,) {
    let mut window = primary_window_query.single_mut();

    window.title = "Ship Designer".to_string();
}

fn setup(
    mut parts: ResMut<Parts>,
    mut mesh_handles: ResMut<PartMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    let part_handles = add_hardcoded_parts(&mut parts);

    for part_handle in part_handles {
        let part = parts.get(&part_handle).unwrap();
        let mesh = part::meshes::mesh_generation::generate_part_mesh(part);
        let mesh_handle = meshes.add(mesh);
        mesh_handles.add(part_handle.id(), mesh_handle);
    }

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert(SelectionSource::new());

    let marker_part_handle = parts.get_handle(PartId::from(1));
    let marker_part = parts.get(&marker_part_handle).unwrap();
    // If we use exactly the part bounds, then we can't place parts next to each other
    let marker_half_extents = marker_part.center() - Vec3::splat(0.01);

    commands.spawn(BuildMarker)
        .insert(BuildMarkerOrientation(Quat::IDENTITY))
        .insert(PbrBundle {
            mesh: part::meshes::get_mesh_or_generate(marker_part_handle.id(), marker_part, &mut mesh_handles, &mut meshes),
            material: materials.add(Color::rgba(0.25, 0.62, 0.26, 0.5).into()),
            ..Default::default()
        })
        .insert(marker_part_handle)
        .insert(Collider::cuboid(
            marker_half_extents.x,
            marker_half_extents.y,
            marker_half_extents.z
        ))
        .insert(Sensor);
}

fn disconnect_on_esc(
    keys: Res<Input<KeyCode>>,
    mut connection_state: ResMut<ConnectionState>
) {
    if keys.pressed(KeyCode::Escape) {
        connection_state.client.disconnect();
    }
}

fn disconnect_on_window_close(
    window_closed: EventReader<WindowClosed>,
    mut connection_state: ResMut<ConnectionState>
) {
    if !window_closed.is_empty() {
        connection_state.client.disconnect();
    }
}