use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Settings {
    pub first_person_sensitivity: f32,
    pub free_camera_sensitivity: f32,
    pub camera_speed: f32,
    pub fullscreen: bool,
    pub draw_camera_gizmo: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            first_person_sensitivity: 0.00025,
            free_camera_sensitivity: 0.00025,
            camera_speed: 1.0,
            fullscreen: false,
            draw_camera_gizmo: false,
        }
    }
}