use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;

use crate::camera::ActiveCamera;

#[derive(Component)]
pub struct SelectionSource {
    intersection: Option<(Entity, RayIntersection)>
}

impl SelectionSource {
    pub fn new() -> Self {
        Self { intersection: None }
    }

    pub fn intersection(&self) -> Option<(Entity, RayIntersection)> {
        self.intersection
    }
}

#[derive(Component)]
pub struct Selectable;

pub fn update_intersections(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut SelectionSource), With<ActiveCamera>>,
    rapier_context: Res<RapierContext>,
    selectable_query: Query<&Selectable>
) {
    let Ok(window) = primary_window_query.get_single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        // Cursor is outside of the window
        return;
    };

    let Ok((camera, camera_transform, mut selection_source)) = camera_query.get_single_mut() else {
        return;
    };

    if let Some(cursor_ray) = camera.viewport_to_world(camera_transform, cursor_position) {
        selection_source.intersection = rapier_context.cast_ray_and_get_normal(
            cursor_ray.origin,
            cursor_ray.direction,
            500.0,
            true,
            QueryFilter::new().exclude_sensors().predicate(&|entity| selectable_query.get(entity).is_ok())
        );
    }
}