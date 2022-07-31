use bevy::ecs::event::EventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::settings::Settings;

#[derive(Default)]
struct Orientation {
    pitch: f32,
    yaw: f32,
}

#[derive(Component)]
pub struct FreeCamera;

// Handle translating the camera's position based on keyboard input
fn camera_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<Settings>,
    mut query: Query<&mut Transform, With<FreeCamera>>
) {
    for mut transform in query.iter_mut() {
        let mut velocity = Vec3::ZERO;

        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity += transform.forward(),
                KeyCode::S => velocity += transform.back(),
                KeyCode::D => velocity += transform.right(),
                KeyCode::A => velocity += transform.left(),
                KeyCode::Space => velocity += Vec3::Y,
                KeyCode::LShift => velocity -= Vec3::Y,
                _ => (),
            }
        }

        velocity = velocity.normalize_or_zero();

        transform.translation += velocity * time.delta_seconds() * settings.camera_speed;
    }
}

fn camera_rotate(
    mut motion_evr: EventReader<MouseMotion>,
    windows: Res<Windows>,
    mut state: ResMut<Orientation>,
    settings: Res<Settings>,
    mut query: Query<&mut Transform, With<FreeCamera>>
) {
    let primary_window = windows.get_primary();
    if let Some(window) = primary_window {
        for mut transform in query.iter_mut() {
            for ev in motion_evr.iter() {
                if window.cursor_locked() {
                    // Why does this work?
                    let window_scale = window.height().min(window.width());
    
                    state.pitch -= (settings.mouse_sensitivity * ev.delta.y * window_scale).to_radians();
                    state.yaw -= (settings.mouse_sensitivity * ev.delta.x * window_scale).to_radians();
                }
    
                transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
            }
        }
    }
}

fn cursor_grab(
    mouse_button_input: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>
) {
    let primary_window = windows.get_primary_mut();
    if let Some(window) = primary_window {
        // Lock and hide the cursor if RMB is pressed
        let rmb_pressed = mouse_button_input.pressed(MouseButton::Right);
        if rmb_pressed && !window.cursor_locked() {
            window.set_cursor_lock_mode(true);
            window.set_cursor_visibility(false);
        } else if !rmb_pressed && window.cursor_locked() {
            window.set_cursor_lock_mode(false);
            window.set_cursor_visibility(true);
        }
    }
}

pub struct FreeCameraPlugin;
impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Orientation>()
            .add_system(camera_move)
            .add_system(camera_rotate)
            .add_system(cursor_grab);
    }
}