use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use bevy_mod_picking::{DefaultPickingPlugins, DebugCursorPickingPlugin, PickableBundle, PickingCameraBundle, RayCastSource, PickingRaycastSet};
use bevy_mod_raycast::IntersectionData;
use bevy_inspector_egui::WorldInspectorPlugin;

use camera::{FreeCameraPlugin, FreeCamera};

pub mod camera;
pub mod settings;

struct PlaceBlockRequest(Entity, IntersectionData);
struct DeleteBlockRequest(Entity);

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(settings::Settings::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(FreeCameraPlugin)
        .add_startup_system(set_window_title)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_event::<PlaceBlockRequest>()
        .add_event::<DeleteBlockRequest>()
        .add_system(build_events)
        .add_system(place_block)
        .add_system(delete_block)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}

fn set_window_title(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_title("Ship Designer".to_string());
}

fn build_events(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut ev_place_block_request: EventWriter<PlaceBlockRequest>,
    mut ev_delete_block_request: EventWriter<DeleteBlockRequest>,
    query: Query<&RayCastSource<PickingRaycastSet>>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let intersection_data = query.iter().next().unwrap().intersect_top();
        if let Some(data) = intersection_data {
            if keys.pressed(KeyCode::LAlt) {
                ev_delete_block_request.send(DeleteBlockRequest(data.0));
            } else {
                ev_place_block_request.send(PlaceBlockRequest(data.0, data.1.clone()));
            }
        }
    }
}

fn place_block(
    mut ev_place_block_requests: EventReader<PlaceBlockRequest>,
    query: Query<&Transform>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for event in ev_place_block_requests.iter() {
        if let Ok(origin_block_transform) = query.get(event.0) {
            let new_block_transform = origin_block_transform.with_translation(origin_block_transform.translation + event.1.normal());

            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                transform: new_block_transform,
                ..Default::default()
            })
            .insert_bundle(PickableBundle::default());
        }
    }
}

fn delete_block(mut ev_delete_block_requests: EventReader<DeleteBlockRequest>, mut commands: Commands) {
    for event in ev_delete_block_requests.iter() {
        commands.entity(event.0).despawn();
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    })
    .insert_bundle(PickableBundle::default());

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert_bundle(PickingCameraBundle::default());
}