use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::fixed_input::FixedInput;

#[derive(Component)]
pub struct ControlledPlayer;

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

    external_impulse.impulse = direction.normalize_or_zero() * 5.0;
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement.in_schedule(CoreSchedule::FixedUpdate));
    }
}