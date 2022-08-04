pub struct Settings {
    pub mouse_sensitivity: f32,
    pub camera_speed: f32,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.0002,
            camera_speed: 15.0,
            fullscreen: false,
        }
    }
}