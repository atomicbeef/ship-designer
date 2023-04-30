use bevy::ecs::event::EventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

use crate::settings::Settings;

#[derive(Default, Resource)]
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
                KeyCode::C => velocity -= Vec3::Y,
                _ => (),
            }
        }

        velocity = velocity.normalize_or_zero();

        transform.translation += velocity * time.delta_seconds() * settings.camera_speed;
    }
}

fn camera_rotate(
    mut motion_evr: EventReader<MouseMotion>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<Orientation>,
    settings: Res<Settings>,
    mut query: Query<&mut Transform, With<FreeCamera>>
) {
    let primary_window = primary_window_query.get_single();
    if let Ok(window) = primary_window {
        for mut transform in query.iter_mut() {
            for ev in motion_evr.iter() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => {},
                    CursorGrabMode::Confined | CursorGrabMode::Locked => {
                        state.pitch -= (settings.mouse_sensitivity * ev.delta.y * window.height()).to_radians();
                        state.yaw -= (settings.mouse_sensitivity * ev.delta.x * window.width()).to_radians();
                    }
                }
    
                transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
            }
        }
    }
}

fn cursor_grab(
    mouse_button_input: Res<Input<MouseButton>>,
    mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_query.get_single_mut();
    if let Ok(mut window) = primary_window {
        // Lock and hide the cursor if RMB is pressed
        let rmb_pressed = mouse_button_input.pressed(MouseButton::Right);
        let cursor_locked = match window.cursor.grab_mode {
            CursorGrabMode::None => false,
            CursorGrabMode::Confined | CursorGrabMode::Locked => true
        };
        
        if rmb_pressed && !cursor_locked {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        } else if !rmb_pressed && cursor_locked {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
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