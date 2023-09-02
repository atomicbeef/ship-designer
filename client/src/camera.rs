use bevy::prelude::*;

#[derive(Resource)]
pub struct ActiveCameraEntity(pub Option<Entity>);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveCameraEntity(None));
    }
}