use bevy::prelude::*;
use bevy::window::{PrimaryWindow, CursorGrabMode};
use bevy_rapier3d::prelude::*;

use common::fixed_update::FixedUpdateSet;
use common::PHYSICS_TIMESTEP;

use crate::camera::ActiveCamera;
use crate::fixed_input::{FixedInput, FixedMouseMotion};
use crate::settings::Settings;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component)]
pub struct ActivelyControlled;

#[derive(Component)]
pub struct PlayerCamera;

fn player_movement(
    keys: Res<FixedInput<KeyCode>>,
    mut player_data_query: Query<(&mut ExternalImpulse, &Transform), (With<LocalPlayer>, With<ActivelyControlled>)>,
    mut motion_reader: EventReader<FixedMouseMotion>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
) {
    let Ok((mut external_impulse, player_transform)) = player_data_query.get_single_mut() else {
        return;
    };

    let Ok(window) = primary_window_query.get_single() else {
        return;
    };
    
    if matches!(window.cursor.grab_mode, CursorGrabMode::None) {
        return;
    } 

    let mut move_direction = Vec3::default();
    let mut rotate_vector = Vec3::default();

    if keys.pressed(KeyCode::W) {
        move_direction += player_transform.forward();
    }

    if keys.pressed(KeyCode::S) {
        move_direction += player_transform.back();
    }

    if keys.pressed(KeyCode::A) {
        move_direction += player_transform.left();
    }

    if keys.pressed(KeyCode::D) {
        move_direction += player_transform.right();
    }

    if keys.pressed(KeyCode::Space) {
        move_direction += player_transform.up();
    }

    if keys.pressed(KeyCode::C) {
        move_direction += player_transform.down();
    }

    if keys.pressed(KeyCode::Q) {
        rotate_vector += player_transform.back() * 5.0;
    }

    if keys.pressed(KeyCode::E) {
        rotate_vector += player_transform.forward() * 5.0;
    }

    let scale_factor = window.height().min(window.width());

    for motion in motion_reader.iter() {
        rotate_vector += player_transform.left() * motion.delta.y * settings.first_person_sensitivity * scale_factor * 12.5;
        rotate_vector += player_transform.down() * motion.delta.x * settings.first_person_sensitivity * scale_factor;
    }

    external_impulse.impulse = move_direction.normalize_or_zero() * PHYSICS_TIMESTEP * 50.0;
    external_impulse.torque_impulse = rotate_vector * PHYSICS_TIMESTEP;
}

fn cursor_lock(
    mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,
    player_camera_query: Query<(), (With<PlayerCamera>, With<ActiveCamera>)>,
    keys: Res<FixedInput<KeyCode>>,
) {
    // Only manage the cursor if the active camera is a player camera
    let Ok(_) = player_camera_query.get_single() else {
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
        ).in_set(FixedUpdateSet::Update));
    }
}