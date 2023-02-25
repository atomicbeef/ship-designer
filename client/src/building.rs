use core::f32::consts::PI;

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use common::colliders::{generate_collider_data, ShapeCollider};
use common::reparent_global_transform::ReparentGlobalTransform;
use uflow::SendMode;
use bevy_rapier3d::prelude::*;

use common::index::Index;
use common::network_id::NetworkId;
use common::compact_transform::CompactTransform;
use common::channels::Channel;
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::materials::Material;
use common::packets::Packet;
use common::shape::{ShapeHandle, Shapes, ShapeId, VOXEL_SIZE};

use crate::building_material::BuildingMaterial;
use crate::connection_state::ConnectionState;
use crate::mesh_generation::{RegenerateShapeMesh, get_mesh_or_generate};
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
    mut marker_query: Query<(&mut Transform, &BuildMarkerOrientation, &ShapeHandle), With<BuildMarker>>,
    body_transform_query: Query<&GlobalTransform>,
    parent_query: Query<&Parent>,
    selection_source_query: Query<&SelectionSource>,
    shapes: Res<Shapes>
) {
    let (mut marker_transform, marker_orientation, shape_handle) = match marker_query.iter_mut().next() {
        Some(x) => x,
        None => { return; }
    };

    if let Some(selection_source) = selection_source_query.iter().next() {
        if let Some((collider, intersection)) = selection_source.intersection() {
            let body = parent_query.get(collider).unwrap().get();

            let shape = shapes.get(shape_handle).unwrap();

            let rotated_center = marker_orientation.0.mul_vec3(shape.center()).abs();
            
            // If the side length is odd, you need to add VOXEL_SIZE / 2.0 as an offset to center it
            let mut odd_offset = Vec3::splat(0.0);
            if shape.width() % 2 == 1 {
                odd_offset.x += VOXEL_SIZE / 2.0;
            }
            if shape.height() % 2 == 1 {
                odd_offset.y += VOXEL_SIZE / 2.0;
            }
            if shape.depth() % 2 == 1 {
                odd_offset.z += VOXEL_SIZE / 2.0;
            }
            odd_offset = marker_orientation.0.mul_vec3(odd_offset).abs();

            let body_transform = body_transform_query.get(body).unwrap();

            // Transform the point to make the calculation easier
            let body_transform_affine = body_transform.affine();
            let body_transform_inverse = body_transform_affine.inverse();

            let inverse_intersection = snap_to_grid(body_transform_inverse.transform_point3(intersection.point), VOXEL_SIZE);
            let inverse_normal = body_transform_inverse.transform_vector3(intersection.normal);

            let inverse_center = inverse_intersection + inverse_normal * rotated_center + odd_offset * (Vec3::splat(1.0) - inverse_normal.abs());

            marker_transform.translation = body_transform_affine.transform_point3(inverse_center);
            
            let (_, body_rotation, _) = body_transform.to_scale_rotation_translation();
            marker_transform.rotation = body_rotation.mul_quat(marker_orientation.0);
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
    mut place_shape_request_writer: EventWriter<PlaceShapeRequest>,
    mut delete_shape_request_writer: EventWriter<DeleteShapeRequest>,
    selection_source_query: Query<&SelectionSource>,
    mut voxel_intersection_query: Query<(&GlobalTransform, &mut ShapeHandle)>,
    mut shapes: ResMut<Shapes>,
    mut regenerate_shape_mesh_writer: EventWriter<RegenerateShapeMesh>,
    parent_query: Query<&Parent>,
    shape_collider_query: Query<&ShapeCollider>,
    ship_transform_query: Query<&GlobalTransform>,
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

    let shape_entity = match shape_collider_query.get(entity) {
        Ok(collider) => collider.shape,
        Err(_) => { return; }
    };

    let body = match parent_query.get(shape_entity) {
        Ok(parent) => parent.get(),
        Err(_) => { return; }
    };

    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Block deletion
        if keys.pressed(KeyCode::LAlt) {
            let network_id = network_id_query.get(shape_entity).unwrap();
            delete_shape_request_writer.send(DeleteShapeRequest(*network_id));
        // Voxel deletion
        } else if keys.pressed(KeyCode::LControl) {
            if let Ok((shape_transform, mut shape_handle)) = voxel_intersection_query.get_mut(shape_entity) {
                let inverse = shape_transform.affine().inverse();
                
                if !inverse.is_finite() {
                    debug!("[Voxel deletion] Uninvertible transform matrix: {}", shape_transform.affine());
                    return;
                }

                let shape = shapes.get_mut(&mut shape_handle).unwrap();

                let inverse_normal = inverse.transform_vector3(intersection_data.normal);
                let inverse_intersection = inverse.transform_point3(intersection_data.point);

                let voxel_pos = (inverse_intersection + shape.center() - inverse_normal * Vec3::splat(VOXEL_SIZE / 2.0)) / VOXEL_SIZE;

                shape.set(voxel_pos.x as u8, voxel_pos.y as u8, voxel_pos.z as u8, Material::Empty);

                regenerate_shape_mesh_writer.send(RegenerateShapeMesh(shape_entity));
            }
        // Block placement
        } else {
            if let Ok(ship_transform) = ship_transform_query.get(body) {
                if let Some((marker_transform, marker_collider)) = marker_query.iter().next() {
                    let (_, marker_rotation, marker_translation) = marker_transform.to_scale_rotation_translation();
                    if rapier_context.intersection_with_shape(
                        marker_translation,
                        marker_rotation,
                        marker_collider,
                        QueryFilter::new().exclude_sensors()
                    ).is_none() {
                        let shape_id = ShapeId::from(1);

                        let ship_space_transform = marker_transform.reparented_to(&ship_transform);
    
                        place_shape_request_writer.send(PlaceShapeRequest {
                            shape_id,
                            shape_transform: CompactTransform::from(ship_space_transform),
                            body_network_id: *network_id_query.get(body).unwrap()
                        });
                    }
                }
            }
        }
    }
}

pub fn send_place_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut place_shape_request_reader: EventReader<PlaceShapeRequest>
) {
    for place_block_request in place_shape_request_reader.iter() {
        let packet: Packet = place_block_request.into();
        connection_state.client.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn send_delete_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_shape_request_reader: EventReader<DeleteShapeRequest>
) {
    for delete_block_request in delete_shape_request_reader.iter() {
        let packet: Packet = delete_block_request.into();
        connection_state.client.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn spawn_shape(
    commands: &mut Commands,
    mesh_handles: &mut MeshHandles,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<BuildingMaterial>,
    shapes: &Shapes,
    shape_handle: ShapeHandle,
    transform: Transform,
    shape_network_id: NetworkId,
    body: Entity,
) -> Entity {
    let shape = shapes.get(&shape_handle).unwrap();

    let mesh_handle = get_mesh_or_generate(shape_handle.id(), shape, mesh_handles, meshes);

    let shape_entity = commands.spawn(MaterialMeshBundle::<BuildingMaterial> {
            mesh: mesh_handle,
            material: materials.add(BuildingMaterial { color: Color::rgb(0.0, 0.3, 0.5).into() }),
            transform,
            ..Default::default()
        })
        .insert(shape_handle)
        .insert(shape_network_id)
        .id();
    
    commands.entity(body).add_child(shape_entity);

    let colliders = generate_collider_data(shape, transform);
    for collider_data in colliders {
        let collider_entity = commands.spawn(collider_data.collider)
            .insert(TransformBundle::from_transform(collider_data.transform))
            .insert(ShapeCollider::new(shape_entity))
            .insert(Selectable)
            .id();
        commands.entity(body).add_child(collider_entity);
    }

    shape_entity
}

pub fn place_shapes(
    mut place_shape_command_reader: EventReader<PlaceShapeCommand>,
    mut commands: Commands,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BuildingMaterial>>,
    shapes: Res<Shapes>,
    network_id_index: Res<Index<NetworkId>>
) {
    for event in place_shape_command_reader.iter() {
        let transform = Transform::from(event.transform);
        let entity = spawn_shape(
            &mut commands,
            &mut mesh_handles,
            &mut meshes,
            &mut materials,
            &shapes,
            shapes.get_handle(event.shape_id),
            transform,
            event.shape_network_id,
            network_id_index.entity(&event.body_network_id).unwrap()
        );
        
        debug!("Spawned shape with entity ID {:?}", entity);
    }
}

pub fn delete_shapes(
    mut delete_shape_command_reader: EventReader<DeleteShapeCommand>,
    mut commands: Commands,
    shape_query: Query<(Entity, &NetworkId)>,
    parent_query: Query<&Parent>,
    ship_children_query: Query<&Children>,
    shape_collider_query: Query<&ShapeCollider>,
) {
    for event in delete_shape_command_reader.iter() {
        for (shape_entity, network_id) in shape_query.iter() {
            if *network_id == event.0 {
                let ship = parent_query.get(shape_entity).unwrap().get();

                // Remove colliders
                let children = ship_children_query.get(ship).unwrap();
                for &child in children {
                    if let Ok(shape_collider) = shape_collider_query.get(child) {
                        if shape_collider.shape == shape_entity {
                            commands.entity(ship).remove_children(&[child]);
                            commands.entity(child).despawn();
                        }
                    }
                }

                // Remove shape
                commands.entity(ship).remove_children(&[shape_entity]);
                commands.entity(shape_entity).despawn();
                debug!("Deleting shape with entity ID {:?}", shape_entity);
            }
        }
    }
}