use bevy::prelude::*;

#[derive(Component)]
pub struct ActiveCamera;

fn cycle_cameras(
    input: Res<Input<KeyCode>>,
    active_camera_query: Query<Entity, With<ActiveCamera>>,
    mut camera_query: Query<(Entity, &mut Camera)>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::F1) {
        if camera_query.iter().count() < 2 {
            return;
        }

        if let Ok(active_camera) = active_camera_query.get_single() {
            // Deactivate old active camera
            commands.entity(active_camera).remove::<ActiveCamera>();
            if let Ok((_, mut camera)) = camera_query.get_mut(active_camera) {
                camera.is_active = false;
            }

            let (new_active_camera, mut camera) = match camera_query.iter_mut()
                .skip_while(|&(entity, _)| entity == active_camera)
                .next() {
                    Some((entity, camera)) => (entity, camera),
                    None => camera_query.iter_mut().next().unwrap(),
                };
            
            commands.entity(new_active_camera).insert(ActiveCamera);
            camera.is_active = true;
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cycle_cameras);
    }
}