use core::f32::consts::PI;

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use common::fixed_update::FixedUpdateSet;
use common::part::colliders::{PartCollider, RegenerateColliders};
use bevy_rapier3d::prelude::*;

use common::network_id::NetworkId;
use common::compact_transform::CompactTransform;
use common::part::events::{PlacePartRequest, DeletePartRequest};
use common::part::materials::Material;
use common::part::{PartHandle, Parts, PartId, VOXEL_SIZE};

use crate::building_material::BuildingMaterial;
use crate::fixed_input::FixedInput;
use crate::part::meshes::{PartMeshHandles, get_mesh_or_generate};
use crate::part::meshes::mesh_generation::RegeneratePartMesh;
use crate::raycast_selection::SelectionSource;

#[derive(Bundle)]
pub struct BuildMarkerBundle {
    pub marker: BuildMarker,
    pub marker_orientation: BuildMarkerOrientation,
    pub pbr_bundle: PbrBundle,
    pub part_handle: PartHandle,
    pub collider: Collider,
    pub sensor: Sensor,
}

impl BuildMarkerBundle {
    pub fn new(
        part_id: PartId,
        parts: &Parts,
        mesh_handles: &mut PartMeshHandles,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>
    ) -> Self {
        let marker_part_handle = parts.get_handle(part_id);
        let marker_part = parts.get(&marker_part_handle).unwrap();
        // If we use exactly the part bounds, then we can't place parts next to each other
        let marker_half_extents = marker_part.center() - Vec3::splat(0.01);

        Self {
            marker: BuildMarker,
            marker_orientation: BuildMarkerOrientation(Quat::IDENTITY),
            pbr_bundle: PbrBundle {
                mesh: get_mesh_or_generate(marker_part_handle.id(), marker_part, mesh_handles, meshes),
                material: materials.add(Color::rgba(0.25, 0.62, 0.26, 0.5).into()),
                ..Default::default()
            },
            part_handle: marker_part_handle,
            collider: Collider::cuboid(
                marker_half_extents.x,
                marker_half_extents.y,
                marker_half_extents.z
            ),
            sensor: Sensor,
        }
    }
}

fn snap_to_grid(point: Vec3, snap_resolution: f32) -> Vec3 {
    // This extra rounding smoothes out any jittering
    let rounded_x = (point.x * 1000.0).round();
    let rounded_y = (point.y * 1000.0).round();
    let rounded_z = (point.z * 1000.0).round();

    let x = (rounded_x * 1.0 / (snap_resolution * 1000.0)).floor() / (1.0 / snap_resolution);
    let y = (rounded_y * 1.0 / (snap_resolution * 1000.0)).floor() / (1.0 / snap_resolution);
    let z = (rounded_z * 1.0 / (snap_resolution * 1000.0)).floor() / (1.0 / snap_resolution);

    Vec3::new(x, y, z)
}

#[derive(Component)]
pub struct BuildMarker;

#[derive(Component)]
pub struct BuildMarkerOrientation(pub Quat);

fn move_build_marker(
    mut marker_query: Query<(&mut Transform, &BuildMarkerOrientation, &PartHandle), With<BuildMarker>>,
    construct_transform_query: Query<&GlobalTransform>,
    parent_query: Query<&Parent>,
    selection_source_query: Query<&SelectionSource>,
    parts: Res<Parts>
) {
    let (mut marker_transform, marker_orientation, part_handle) = match marker_query.iter_mut().next() {
        Some(x) => x,
        None => { return; }
    };

    if let Some(selection_source) = selection_source_query.iter().next() {
        if let Some((collider, intersection)) = selection_source.intersection() {
            let construct = parent_query.get(collider).unwrap().get();

            let part = parts.get(part_handle).unwrap();

            let rotated_center = marker_orientation.0.mul_vec3(part.center()).abs();
            
            // If the side length is odd, you need to add VOXEL_SIZE / 2.0 as an offset to center it
            let mut odd_offset = Vec3::splat(0.0);
            if part.width() % 2 == 1 {
                odd_offset.x += VOXEL_SIZE / 2.0;
            }
            if part.height() % 2 == 1 {
                odd_offset.y += VOXEL_SIZE / 2.0;
            }
            if part.depth() % 2 == 1 {
                odd_offset.z += VOXEL_SIZE / 2.0;
            }
            odd_offset = marker_orientation.0.mul_vec3(odd_offset).abs();

            let construct_transform = construct_transform_query.get(construct).unwrap();

            // Transform the point to make the calculation easier
            let construct_transform_affine = construct_transform.affine();
            let construct_transform_inverse = construct_transform_affine.inverse();

            let inverse_intersection = snap_to_grid(construct_transform_inverse.transform_point3(intersection.point), VOXEL_SIZE);
            let inverse_normal = construct_transform_inverse.transform_vector3(intersection.normal);

            let inverse_center = inverse_intersection + inverse_normal * rotated_center + odd_offset * (Vec3::splat(1.0) - inverse_normal.abs());

            marker_transform.translation = construct_transform_affine.transform_point3(inverse_center);
            
            let (_, construct_rotation, _) = construct_transform.to_scale_rotation_translation();
            marker_transform.rotation = construct_rotation.mul_quat(marker_orientation.0);
        }
    }
}

fn rotate_build_marker(
    mut marker_query: Query<&mut BuildMarkerOrientation, With<BuildMarker>>,
    keys: Res<FixedInput<KeyCode>>
) {
    let mut marker_orientation = match marker_query.iter_mut().next() {
        Some(transform) => transform,
        None => { return; }
    };

    if keys.just_pressed(KeyCode::J) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_x(PI / 2.0));
    }

    if keys.just_pressed(KeyCode::U) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_x(-PI / 2.0));
    }

    if keys.just_pressed(KeyCode::K) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_y(PI / 2.0));
    }

    if keys.just_pressed(KeyCode::I) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_y(-PI / 2.0));
    }

    if keys.just_pressed(KeyCode::L) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_z(PI / 2.0));
    }

    if keys.just_pressed(KeyCode::O) {
        marker_orientation.0 = marker_orientation.0.mul_quat(Quat::from_rotation_z(-PI / 2.0));
    }
}

fn create_build_request_events(
    mouse_buttons: Res<FixedInput<MouseButton>>,
    keys: Res<FixedInput<KeyCode>>,
    mut place_part_request_writer: EventWriter<PlacePartRequest>,
    mut delete_part_request_writer: EventWriter<DeletePartRequest>,
    selection_source_query: Query<&SelectionSource>,
    mut voxel_intersection_query: Query<(&GlobalTransform, &mut PartHandle)>,
    mut parts: ResMut<Parts>,
    mut regenerate_part_mesh_writer: EventWriter<RegeneratePartMesh>,
    mut regenerate_collider_writer: EventWriter<RegenerateColliders>,
    parent_query: Query<&Parent>,
    part_collider_query: Query<&PartCollider>,
    construct_transform_query: Query<&GlobalTransform>,
    marker_query: Query<(&GlobalTransform, &Collider), With<BuildMarker>>,
    network_id_query: Query<&NetworkId>,
    rapier_context: Res<RapierContext>
) {
    let (entity, intersection_data) = match selection_source_query.iter().next() {
        Some(source) => match source.intersection() {
            Some(data) => data,
            None => { return; }
        },
        None => { return; }
    };

    let part_entity = match part_collider_query.get(entity) {
        Ok(collider) => collider.part,
        Err(_) => { return; }
    };

    let construct = match parent_query.get(part_entity) {
        Ok(parent) => parent.get(),
        Err(_) => { return; }
    };

    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Part deletion
        if keys.pressed(KeyCode::AltLeft) {
            let network_id = network_id_query.get(part_entity).unwrap();
            delete_part_request_writer.send(DeletePartRequest(*network_id));
        // Voxel deletion
        } else if keys.pressed(KeyCode::ControlLeft) {
            if let Ok((part_transform, mut part_handle)) = voxel_intersection_query.get_mut(part_entity) {
                let inverse = part_transform.affine().inverse();
                
                if !inverse.is_finite() {
                    debug!("[Voxel deletion] Uninvertible transform matrix: {}", part_transform.affine());
                    return;
                }

                let part = parts.get_mut(&mut part_handle).unwrap();

                let inverse_normal = inverse.transform_vector3(intersection_data.normal);
                let inverse_intersection = inverse.transform_point3(intersection_data.point);
                
                let voxel_pos = (inverse_intersection + part.center() - inverse_normal * Vec3::splat(VOXEL_SIZE / 2.0)) / VOXEL_SIZE;

                part.set(voxel_pos.into(), Material::Empty);

                regenerate_part_mesh_writer.send(RegeneratePartMesh(part_entity));
                regenerate_collider_writer.send(RegenerateColliders(part_entity));
            }
        // Part placement
        } else {
            if let Ok(construct_transform) = construct_transform_query.get(construct) {
                if let Some((marker_transform, marker_collider)) = marker_query.iter().next() {
                    let (_, marker_rotation, marker_translation) = marker_transform.to_scale_rotation_translation();
                    if rapier_context.intersection_with_shape(
                        marker_translation,
                        marker_rotation,
                        marker_collider,
                        QueryFilter::new().exclude_sensors()
                    ).is_none() {
                        let part_id = PartId::from(1);

                        let construct_space_transform = marker_transform.reparented_to(&construct_transform);
    
                        place_part_request_writer.send(PlacePartRequest {
                            part_id,
                            part_transform: CompactTransform::from(construct_space_transform),
                            construct_network_id: *network_id_query.get(construct).unwrap()
                        });
                    }
                }
            }
        }
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BuildingMaterial>::default())
            .add_systems(FixedUpdate, (
                move_build_marker,
                rotate_build_marker,
                create_build_request_events
            ).chain().in_set(FixedUpdateSet::Update));
    }
}