use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy::input::mouse::MouseButton;
use bevy_mod_picking::{DefaultPickingPlugins, DebugCursorPickingPlugin, PickableBundle, PickingCameraBundle, RayCastSource, PickingRaycastSet};

use camera::{FreeCameraPlugin, FreeCamera};

pub mod camera;
pub mod settings;

struct PlaceBlockRequest(Entity, GridPos);
struct DeleteBlockRequest(Entity, GridPos);

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct GridPos {
    x: i32,
    y: i32,
    z: i32
}

impl From<Transform> for GridPos {
    fn from(transform: Transform) -> Self {
        GridPos {
            x: transform.translation.x as i32,
            y: transform.translation.y as i32,
            z: transform.translation.z as i32
        }
    }
}

impl From<&Transform> for GridPos {
    fn from(transform: &Transform) -> Self {
        GridPos {
            x: transform.translation.x as i32,
            y: transform.translation.y as i32,
            z: transform.translation.z as i32
        }
    }
}

impl From<Vec3> for GridPos {
    fn from(vec3: Vec3) -> Self {
        GridPos {
            x: vec3.x as i32,
            y: vec3.y as i32,
            z: vec3.z as i32
        }
    }
}

#[derive(Component)]
struct ShipData(HashMap<GridPos, Entity>);

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(settings::Settings::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(FreeCameraPlugin)
        .add_startup_system(set_window_title)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_event::<PlaceBlockRequest>()
        .add_event::<DeleteBlockRequest>()
        .add_system(build_events)
        .add_system(place_block)
        .add_system(delete_block)
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
    intersection_query: Query<&RayCastSource<PickingRaycastSet>>,
    transform_query: Query<&Transform>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let intersection_data = intersection_query.iter().next().unwrap().intersect_top();
        if let Some(data) = intersection_data {
            // Block deletion
            if keys.pressed(KeyCode::LAlt) {
                if let Ok(block_transform) = transform_query.get(data.0) {
                    let block_pos = block_transform.into();

                    ev_delete_block_request.send(DeleteBlockRequest(data.0, block_pos));
                }
            // Block placement
            } else {
                if let Ok(origin_block_transform) = transform_query.get(data.0) {
                    let block_pos = (origin_block_transform.translation + data.1.normal()).into();

                    ev_place_block_request.send(PlaceBlockRequest(data.0, block_pos));
                }
            }
        }
    }
}

fn place_block(
    mut ev_place_block_requests: EventReader<PlaceBlockRequest>,
    mut ship_data_query: Query<&mut ShipData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for event in ev_place_block_requests.iter() {
        let block_id = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform { translation: Vec3::new(event.1.x as f32, event.1.y as f32, event.1.z as f32), ..Default::default() },
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .id();

        if let Some(mut ship_data) = ship_data_query.iter_mut().next() {
            ship_data.0.insert(event.1, block_id);
        }
    }
}

fn delete_block(
    mut ev_delete_block_requests: EventReader<DeleteBlockRequest>,
    mut commands: Commands,
    mut ship_data_query: Query<&mut ShipData>
) {
    for event in ev_delete_block_requests.iter() {
        commands.entity(event.0).despawn();

        if let Some(mut ship_data) = ship_data_query.iter_mut().next() {
            ship_data.0.remove(&event.1);
        }
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

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FreeCamera)
    .insert_bundle(PickingCameraBundle::default());

    commands.spawn().insert(ShipData(HashMap::new()));
}