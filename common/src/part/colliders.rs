use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;

use crate::materials::Material;
use crate::part::{Part, PartHandle, VOXEL_SIZE};

#[derive(Clone, Copy, Component, PartialEq, Eq, Hash)]
pub struct PartCollider {
    pub part: Entity
}

impl PartCollider {
    pub fn new(part: Entity) -> Self {
        Self { part }
    }
}

#[derive(Debug)]
pub struct ColliderData {
    pub collider: Collider,
    pub transform: Transform
}

pub fn generate_collider_data(
    part: &Part,
    part_transform: Transform
) -> Vec<ColliderData> {
    let mut colliders = Vec::new();
    let mut tested = vec![false; part.size() as usize];

    for start_z in 0..part.depth() {
        for start_y in 0..part.height() {
            for start_x in 0..part.width() {
                let start_index = part.pos_to_index(start_x, start_y, start_z);
                if tested[start_index] {
                    continue; 
                }

                let material = part.get(start_x, start_y, start_z);

                if matches!(material, Material::Empty) {
                    tested[start_index] = true;
                    continue;
                }

                tested[start_index] = true;

                let mut end_x = start_x;
                let mut end_y = start_y;
                let mut end_z = start_z;

                for x in start_x + 1..part.width() {
                    let current_index = part.pos_to_index(x, start_y, start_z);
                    let test_material = part.get_index(current_index);

                    if test_material != material || tested[current_index] {
                        end_x = x - 1;
                        break;
                    }

                    if x == part.width() - 1 {
                        end_x = x;
                    }

                    tested[current_index] = true;
                }

                'height: for y in start_y + 1..part.height() {
                    for x in start_x..end_x + 1 {
                        let current_index = part.pos_to_index(x, y, start_z);
                        let test_material = part.get_index(current_index);

                        if test_material != material || tested[current_index] {
                            end_y = y - 1;
                            break 'height;
                        }
                    }

                    for x in start_x..end_x + 1 {
                        tested[part.pos_to_index(x, y, start_z)] = true;
                    }

                    if y == part.height() - 1 {
                        end_y = y;
                    }
                }

                'depth: for z in start_z + 1..part.depth() {
                    for y in start_y..end_y + 1 {
                        for x in start_x..end_x + 1 {
                            let current_index = part.pos_to_index(x, y, z);
                            let test_material = part.get_index(current_index);
                            if test_material != material || tested[current_index] {
                                end_z = z - 1;
                                break 'depth;
                            }
                        }
                    }

                    for y in start_y..end_y + 1 {
                        for x in start_x..end_x + 1 {
                            tested[part.pos_to_index(x, y, z)] = true;
                        }
                    }

                    if z == part.depth() - 1 {
                        end_z = z;
                    }
                }

                let hx = (end_x + 1 - start_x) as f32 / 2.0 * VOXEL_SIZE;
                let hy = (end_y + 1 - start_y) as f32 / 2.0 * VOXEL_SIZE;
                let hz = (end_z + 1 - start_z) as f32 / 2.0 * VOXEL_SIZE;

                let collider = Collider::cuboid(hx, hy, hz);

                let part_space_transform = Transform {
                    translation: Vec3::new(start_x as f32, start_y as f32, start_z as f32) * VOXEL_SIZE + Vec3::new(hx, hy, hz),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(1.0)
                };

                let uncentered_part_transform = Transform {
                    translation: part_transform.translation - part_transform.rotation.mul_vec3(part.center()),
                    rotation: part_transform.rotation,
                    scale: part_space_transform.scale
                };

                let transform = uncentered_part_transform.mul_transform(part_space_transform);

                colliders.push(ColliderData { collider, transform });
            }
        }
    }

    colliders
}

pub struct RegenerateColliders(pub Entity);

pub fn remove_unused_colliders(
    mut removed_parts: RemovedComponents<PartHandle>,
    collider_query: Query<(Entity, &PartCollider)>,
    parent_query: Query<&Parent>,
    mut commands: Commands
) {
    for entity in removed_parts.iter() {
        for (collider, part_collider) in collider_query.iter() {
            if part_collider.part == entity {
                if let Ok(parent) = parent_query.get(collider) {
                    commands.entity(parent.get()).remove_children(&[collider]);
                }
                
                commands.entity(collider).despawn();
            }
        }
    }
}