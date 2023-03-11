use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use crate::materials::Material;
use crate::shape::{Shape, ShapeHandle, VOXEL_SIZE};

#[derive(Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct ShapeCollider {
    pub shape: Entity
}

impl ShapeCollider {
    pub fn new(shape: Entity) -> Self {
        Self { shape }
    }
}

#[derive(Debug)]
pub struct ColliderData {
    pub collider: Collider,
    pub transform: Transform
}

pub fn generate_collider_data(
    shape: &Shape,
    shape_transform: Transform
) -> Vec<ColliderData> {
    let mut colliders = Vec::new();
    let mut tested = vec![false; shape.size() as usize];

    for start_z in 0..shape.depth() {
        for start_y in 0..shape.height() {
            for start_x in 0..shape.width() {
                let start_index = shape.pos_to_index(start_x, start_y, start_z);
                if tested[start_index] {
                    continue; 
                }

                let material = shape.get(start_x, start_y, start_z);

                if matches!(material, Material::Empty) {
                    tested[start_index] = true;
                    continue;
                }

                tested[start_index] = true;

                let mut end_x = start_x;
                let mut end_y = start_y;
                let mut end_z = start_z;

                for x in start_x + 1..shape.width() {
                    let current_index = shape.pos_to_index(x, start_y, start_z);
                    let test_material = shape.get_index(current_index);

                    if test_material != material || tested[current_index] {
                        end_x = x - 1;
                        break;
                    }

                    if x == shape.width() - 1 {
                        end_x = x;
                    }

                    tested[current_index] = true;
                }

                'height: for y in start_y + 1..shape.height() {
                    for x in start_x..end_x + 1 {
                        let current_index = shape.pos_to_index(x, y, start_z);
                        let test_material = shape.get_index(current_index);

                        if test_material != material || tested[current_index] {
                            end_y = y - 1;
                            break 'height;
                        }
                    }

                    for x in start_x..end_x + 1 {
                        tested[shape.pos_to_index(x, y, start_z)] = true;
                    }

                    if y == shape.height() - 1 {
                        end_y = y;
                    }
                }

                'depth: for z in start_z + 1..shape.depth() {
                    for y in start_y..end_y + 1 {
                        for x in start_x..end_x + 1 {
                            let current_index = shape.pos_to_index(x, y, z);
                            let test_material = shape.get_index(current_index);
                            if test_material != material || tested[current_index] {
                                end_z = z - 1;
                                break 'depth;
                            }
                        }
                    }

                    for y in start_y..end_y + 1 {
                        for x in start_x..end_x + 1 {
                            tested[shape.pos_to_index(x, y, z)] = true;
                        }
                    }

                    if z == shape.depth() - 1 {
                        end_z = z;
                    }
                }

                let hx = (end_x + 1 - start_x) as f32 / 2.0 * VOXEL_SIZE;
                let hy = (end_y + 1 - start_y) as f32 / 2.0 * VOXEL_SIZE;
                let hz = (end_z + 1 - start_z) as f32 / 2.0 * VOXEL_SIZE;

                let collider = Collider::cuboid(hx, hy, hz);
                let shape_corner = shape_transform.translation - shape.center();
                let collider_corner = shape_corner + Vec3::new(start_x as f32, start_y as f32, start_z as f32) * VOXEL_SIZE;
                let collider_translation = collider_corner + Vec3::new(hx, hy, hz);

                let transform = Transform {
                    translation: collider_translation,
                    rotation: shape_transform.rotation,
                    scale: Vec3::splat(1.0)
                };

                colliders.push(ColliderData { collider, transform });
            }
        }
    }

    colliders
}

pub struct RegenerateColliders(pub Entity);

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