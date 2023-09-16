use bevy::prelude::*;

use crate::player_controller::ActivelyControlled;
use crate::settings::Settings;

#[derive(Component)]
pub struct ActiveCamera;

fn cycle_cameras(
    input: Res<Input<KeyCode>>,
    mut active_camera_query: Query<(Entity, Option<&Parent>), With<ActiveCamera>>,
    mut camera_query: Query<(Entity, Option<&Parent>, &mut Camera)>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::F1) {
        if camera_query.iter().count() < 2 {
            return;
        }

        if let Ok((active_camera, potential_player)) = active_camera_query.get_single_mut() {
            // Deactivate old active camera
            commands.entity(active_camera).remove::<ActiveCamera>();
            if let Ok((_, _, mut camera)) = camera_query.get_mut(active_camera) {
                camera.is_active = false;
            }
            
            if let Some(player) = potential_player {
                commands.entity(player.get()).remove::<ActivelyControlled>();
            }

            let (new_active_camera, new_potential_player, mut camera) = match camera_query.iter_mut()
                .skip_while(|&(entity, _, _)| entity == active_camera)
                .next() {
                    Some((entity, maybe_player, camera)) => (entity, maybe_player, camera),
                    None => camera_query.iter_mut().next().unwrap(),
                };
            
            commands.entity(new_active_camera).insert(ActiveCamera);
            camera.is_active = true;

            if let Some(player) = new_potential_player {
                commands.entity(player.get()).insert(ActivelyControlled);
            }
        }
    }
}

fn draw_cameras(
    camera_transform_query: Query<&GlobalTransform, (With<Camera>, Without<ActiveCamera>)>,
    mut gizmos: Gizmos,
    settings: Res<Settings>,
) {
    if !settings.draw_debug {
        return;
    }

    for camera_transform in camera_transform_query.iter() {
        gizmos.sphere(camera_transform.translation(), Quat::IDENTITY, 0.2, Color::GREEN);
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            cycle_cameras,
        ));
    }
}

pub struct CameraDebugPlugin;

impl Plugin for CameraDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            draw_cameras,
        ));
    }
}