use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Settings {
    pub mouse_sensitivity: f32,
    pub camera_speed: f32,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.00025,
            camera_speed: 1.0,
            fullscreen: false,
        }
    }
}