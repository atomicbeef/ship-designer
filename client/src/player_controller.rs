use bevy::prelude::*;
use bevy::window::{PrimaryWindow, CursorGrabMode};
use bevy_rapier3d::prelude::*;
use common::fixed_update::FixedUpdateSet;

use crate::camera::ActiveCameraEntity;
use crate::fixed_input::{FixedInput, FixedMouseMotion};
use crate::settings::Settings;

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Component)]
pub struct PlayerCamera;

fn player_movement(
    keys: Res<FixedInput<KeyCode>>,
    mut external_impulse_query: Query<&mut ExternalImpulse, With<ControlledPlayer>>,
) {
    let mut external_impulse = match external_impulse_query.iter_mut().next() {
        Some(query) => query,
        None => { return; },
    };

    let mut direction = Vec3::default();

    if keys.pressed(KeyCode::W) {
        direction.z -= 1.0;
    }

    if keys.pressed(KeyCode::S) {
        direction.z += 1.0;
    }

    if keys.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }

    if keys.pressed(KeyCode::D) {
        direction.x += 1.0;
    }

    if keys.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }

    if keys.pressed(KeyCode::C) {
        direction.y -= 1.0;
    }

    external_impulse.impulse = direction.normalize_or_zero() * common::PHYSICS_TIMESTEP * 50.0;
}

fn player_rotation(
    mut motion_reader: EventReader<FixedMouseMotion>,
    mut player_transform_query: Query<&mut Transform, With<ControlledPlayer>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
) {
    let Ok(window) = primary_window_query.get_single() else {
        return;
    };

    for motion in motion_reader.iter() {
        let Ok(mut transform) = player_transform_query.get_single_mut() else {
            return;
        };

        let scale_factor = window.height().min(window.width());
        let yaw = -motion.delta.x * settings.first_person_sensitivity * scale_factor;
        let pitch = -motion.delta.y * settings.first_person_sensitivity * scale_factor;

        transform.rotate_y(yaw.to_radians());
        transform.rotate_local_x(pitch.to_radians());
    }
}

fn cursor_lock(
    mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,
    active_camera: Res<ActiveCameraEntity>,
    player_camera_query: Query<(), With<PlayerCamera>>,
    keys: Res<FixedInput<KeyCode>>,
) {
    // Only manage the cursor if the active camera is a player camera
    let Some(camera_entity) = active_camera.0 else {
        return;
    };
    let Ok(_) = player_camera_query.get(camera_entity) else {
        return;
    };

    if let Ok(mut window) = primary_window_query.get_single_mut() {
        if keys.just_pressed(KeyCode::Tab) {
            let cursor_locked = match window.cursor.grab_mode {
                CursorGrabMode::None => false,
                CursorGrabMode::Confined | CursorGrabMode::Locked => true
            };
            
            if !cursor_locked {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            } else {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }
    }
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            cursor_lock,
            player_movement,
            player_rotation,
        ).in_set(FixedUpdateSet::Update));
    }
}