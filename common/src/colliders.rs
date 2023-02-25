use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use crate::shape::{Shape, ShapeHandle};

#[derive(Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct ShapeCollider {
    pub shape: Entity
}

impl ShapeCollider {
    pub fn new(shape: Entity) -> Self {
        Self { shape }
    }
}

pub struct ColliderData {
    pub collider: Collider,
    pub transform: Transform
}

pub fn generate_collider_data(
    shape: &Shape,
    shape_transform: Transform
) -> Vec<ColliderData> {
    let mut colliders = Vec::new();

    let center = shape.center();
    let collider = Collider::cuboid(center.x, center.y, center.z);
    colliders.push(ColliderData { collider, transform: shape_transform });

    colliders
}

pub fn remove_unused_colliders(
    removed_shapes: RemovedComponents<ShapeHandle>,
    collider_query: Query<(Entity, &ShapeCollider)>,
    parent_query: Query<&Parent>,
    mut commands: Commands
) {
    for entity in removed_shapes.iter() {
        for (collider, shape_collider) in collider_query.iter() {
            if shape_collider.shape == entity {
                if let Ok(parent) = parent_query.get(collider) {
                    commands.entity(parent.get()).remove_children(&[collider]);
                }
                
                commands.entity(collider).despawn();
            }
        }
    }
}