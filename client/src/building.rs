use core::f32::consts::PI;

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use common::colliders::{generate_collider_data, PartCollider, RegenerateColliders};
use uflow::SendMode;
use bevy_rapier3d::prelude::*;

use common::index::Index;
use common::network_id::NetworkId;
use common::compact_transform::CompactTransform;
use common::channels::Channel;
use common::events::building::{PlacePartRequest, PlacePartCommand, DeletePartRequest, DeletePartCommand};
use common::materials::Material;
use common::packets::Packet;
use common::part::{PartHandle, Parts, PartId, VOXEL_SIZE};

use crate::building_material::BuildingMaterial;
use crate::connection_state::ConnectionState;
use crate::mesh_generation::{RegeneratePartMesh, get_mesh_or_generate};
use crate::meshes::MeshHandles;
use crate::raycast_selection::{SelectionSource, Selectable};

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

pub fn move_build_marker(
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

pub fn rotate_build_marker(
    mut marker_query: Query<&mut BuildMarkerOrientation, With<BuildMarker>>,
    keys: Res<Input<KeyCode>>
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

pub fn build_request_events(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
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
        if keys.pressed(KeyCode::LAlt) {
            let network_id = network_id_query.get(part_entity).unwrap();
            delete_part_request_writer.send(DeletePartRequest(*network_id));
        // Voxel deletion
        } else if keys.pressed(KeyCode::LControl) {
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

                part.set(voxel_pos.x as u8, voxel_pos.y as u8, voxel_pos.z as u8, Material::Empty);

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

pub fn send_place_part_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut place_part_request_reader: EventReader<PlacePartRequest>
) {
    for place_part_request in place_part_request_reader.iter() {
        let packet: Packet = place_part_request.into();
        connection_state.client.send((&packet).into(), Channel::PartCommands.into(), SendMode::Reliable);
    }
}

pub fn send_delete_part_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_part_request_reader: EventReader<DeletePartRequest>
) {
    for delete_part_request in delete_part_request_reader.iter() {
        let packet: Packet = delete_part_request.into();
        connection_state.client.send((&packet).into(), Channel::PartCommands.into(), SendMode::Reliable);
    }
}

pub fn spawn_part(
    commands: &mut Commands,
    mesh_handles: &mut MeshHandles,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<BuildingMaterial>,
    parts: &Parts,
    part_handle: PartHandle,
    transform: Transform,
    part_network_id: NetworkId,
    construct: Entity,
) -> Entity {
    let part = parts.get(&part_handle).unwrap();

    let mesh_handle = get_mesh_or_generate(part_handle.id(), part, mesh_handles, meshes);

    let part_entity = commands.spawn(MaterialMeshBundle::<BuildingMaterial> {
            mesh: mesh_handle,
            material: materials.add(BuildingMaterial { color: Color::rgb(0.0, 0.3, 0.5).into() }),
            transform,
            ..Default::default()
        })
        .insert(part_handle)
        .insert(part_network_id)
        .id();
    
    commands.entity(construct).add_child(part_entity);

    let colliders = generate_collider_data(part, transform);
    for collider_data in colliders {
        let collider_entity = commands.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(PartCollider::new(part_entity))
            .insert(Selectable)
            .id();
        commands.entity(construct).add_child(collider_entity);
    }

    part_entity
}

pub fn place_parts(
    mut place_part_command_reader: EventReader<PlacePartCommand>,
    mut commands: Commands,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BuildingMaterial>>,
    parts: Res<Parts>,
    network_id_index: Res<Index<NetworkId>>
) {
    for event in place_part_command_reader.iter() {
        let transform = Transform::from(event.transform);
        let entity = spawn_part(
            &mut commands,
            &mut mesh_handles,
            &mut meshes,
            &mut materials,
            &parts,
            parts.get_handle(event.part_id),
            transform,
            event.part_network_id,
            network_id_index.entity(&event.construct_network_id).unwrap()
        );
        
        debug!("Spawned part with entity ID {:?}", entity);
    }
}

pub fn delete_parts(
    mut delete_part_command_reader: EventReader<DeletePartCommand>,
    mut commands: Commands,
    part_query: Query<(Entity, &NetworkId)>,
    parent_query: Query<&Parent>,
    construct_children_query: Query<&Children>,
    part_collider_query: Query<&PartCollider>,
) {
    for event in delete_part_command_reader.iter() {
        for (part_entity, network_id) in part_query.iter() {
            if *network_id == event.0 {
                let construct = parent_query.get(part_entity).unwrap().get();

                // Remove colliders
                let children = construct_children_query.get(construct).unwrap();
                for &child in children {
                    if let Ok(part_collider) = part_collider_query.get(child) {
                        if part_collider.part == part_entity {
                            commands.entity(construct).remove_children(&[child]);
                            commands.entity(child).despawn();
                        }
                    }
                }

                // Remove part
                commands.entity(construct).remove_children(&[part_entity]);
                commands.entity(part_entity).despawn();
                debug!("Deleting part with entity ID {:?}", part_entity);
            }
        }
    }
}

pub fn regenerate_colliders(
    mut commands: Commands,
    mut regenerate_colliders_reader: EventReader<RegenerateColliders>,
    parent_query: Query<&Parent, With<PartHandle>>,
    children_query: Query<&Children>,
    part_colliders_query: Query<&PartCollider>,
    part_query: Query<(&PartHandle, &Transform)>,
    parts: Res<Parts>
) {
    for request in regenerate_colliders_reader.iter() {
        let (part_handle, transform) = part_query.get(request.0).unwrap();
        let part = parts.get(&part_handle).unwrap();

        if let Ok(parent) = parent_query.get(request.0) {
            let construct = parent.get();

            // Delete old colliders
            for &child in children_query.get(construct).unwrap() {
                if let Ok(part_collider) = part_colliders_query.get(child) {
                    if part_collider.part == request.0 {
                        commands.entity(construct).remove_children(&[child]);
                        commands.entity(child).despawn();
                    }
                }
            }

            // Spawn new colliders
            let colliders = generate_collider_data(part, *transform);
            for collider_data in colliders {
                let collider_entity = commands.spawn(collider_data.collider)
                    .insert(TransformBundle::from_transform(collider_data.transform))
                    .insert(PartCollider::new(request.0))
                    .insert(Selectable)
                    .id();
                commands.entity(construct).add_child(collider_entity);
            }
        }
    }
}