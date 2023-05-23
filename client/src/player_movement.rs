use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::fixed_input::FixedInput;

fn player_movement(
    keys: Res<FixedInput<KeyCode>>,
) {
    
}

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement.in_schedule(CoreSchedule::FixedUpdate));
    }
}