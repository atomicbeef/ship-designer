use bevy::input::mouse::MouseButton;
use bevy_mod_picking::{PickableBundle, PickingRaycastSet, RaycastSource};
use bevy::prelude::*;
use uflow::SendMode;
use bevy_rapier3d::prelude::*;

use common::network_id::NetworkId;
use common::shape_transform::ShapeTransform;
use common::channels::Channel;
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::materials::Material;
use common::packets::Packet;
use common::shape::{ShapeHandle, Shapes, ShapeId, VOXEL_SIZE};

use crate::connection_state::ConnectionState;
use crate::mesh_generation::{RegenerateShapeMesh, generate_shape_mesh};
use crate::meshes::MeshHandles;

pub fn build_request_events(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut place_shape_request_writer: EventWriter<PlaceShapeRequest>,
    mut delete_shape_request_writer: EventWriter<DeleteShapeRequest>,
    intersection_query: Query<&RaycastSource<PickingRaycastSet>>,
    mut voxel_intersection_query: Query<(&GlobalTransform, &mut ShapeHandle)>,
    mut shapes: ResMut<Shapes>,
    mut regenerate_shape_mesh_writer: EventWriter<RegenerateShapeMesh>,
    transform_query: Query<&Transform>,
    network_id_query: Query<&NetworkId>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let intersection_data = intersection_query.iter().next().unwrap().get_nearest_intersection();
        if let Some((entity, data)) = intersection_data {
            // Block deletion
            if keys.pressed(KeyCode::LAlt) {
                let network_id = network_id_query.get(entity).unwrap();
                delete_shape_request_writer.send(DeleteShapeRequest(*network_id));
            // Voxel deletion
            } else if keys.pressed(KeyCode::LControl) {
                if let Ok((shape_transform, mut shape_handle)) = voxel_intersection_query.get_mut(entity) {
                    let transform_affine = shape_transform.affine();
                    let inverse = shape_transform.affine().inverse();
                    
                    let transform_matrix;
                    if inverse.is_finite() {
                        transform_matrix = inverse;
                    } else {
                        debug!("[Voxel deletion] Uninvertible transform matrix: {}", transform_affine);
                        transform_matrix = transform_affine;
                    }

                    let voxel_pos = transform_matrix.transform_point3(data.position());
                    debug!(?data);
                    debug!(?voxel_pos);
                    
                    let shape = shapes.get_mut(&mut shape_handle).unwrap();

                    shape.set(voxel_pos.x as u8, voxel_pos.y as u8, voxel_pos.z as u8, Material::Empty);

                    regenerate_shape_mesh_writer.send(RegenerateShapeMesh(entity));
                }
            // Block placement
            } else {
                if let Ok(origin_shape_transform) = transform_query.get(entity) {
                    let shape_id = ShapeId::from(0);
                    let translation = origin_shape_transform.translation + data.normal();
                    let transform = ShapeTransform::from_xyz(
                        translation.x as i16,
                        translation.y as i16,
                        translation.z as i16
                    );

                    place_shape_request_writer.send(PlaceShapeRequest(shape_id, transform));
                }
            }
        }
    }
}

pub fn send_place_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut place_block_request_reader: EventReader<PlaceShapeRequest>
) {
    for place_block_request in place_block_request_reader.iter() {
        let packet: Packet = place_block_request.into();
        connection_state.client.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn send_delete_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_block_request_reader: EventReader<DeleteShapeRequest>
) {
    for delete_block_request in delete_block_request_reader.iter() {
        let packet: Packet = delete_block_request.into();
        connection_state.client.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn spawn_shape(
    commands: &mut Commands,
    mesh_handles: &mut MeshHandles,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    shapes: &Shapes,
    shape_handle: ShapeHandle,
    transform: Transform,
    network_id: NetworkId
) -> Entity {
    let shape = shapes.get(&shape_handle).unwrap();

    let mesh_handle = match mesh_handles.get(&shape_handle) {
        Some(mesh_handle) => mesh_handle.clone(),
        None => {
            let mesh = generate_shape_mesh(shape);

            let mesh_handle = meshes.add(mesh);
            mesh_handles.add(shape_handle.clone(), mesh_handle.clone());

            mesh_handle
        }
    };

    commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: transform,
            ..Default::default()
        })
        .insert(shape_handle)
        .insert(network_id)
        .insert(Collider::cuboid(
            shape.width() as f32 * VOXEL_SIZE / 2.0, 
            shape.height() as f32 * VOXEL_SIZE / 2.0,
            shape.depth() as f32 * VOXEL_SIZE / 2.0
        ))
        .insert(PickableBundle::default())
        .id()
}

pub fn place_shapes(
    mut place_shape_command_reader: EventReader<PlaceShapeCommand>,
    mut commands: Commands,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shapes: Res<Shapes>
) {
    for event in place_shape_command_reader.iter() {
        let transform = Transform::from(event.transform);
        let entity = spawn_shape(
            &mut commands,
            &mut mesh_handles,
            &mut meshes,
            &mut materials,
            &shapes,
            ShapeHandle::new(ShapeId::from(0)),
            transform,
            event.network_id
        );
        
        dbg!("Spawned shape with entity ID", entity);
    }
}

pub fn delete_shapes(
    mut delete_shape_command_reader: EventReader<DeleteShapeCommand>,
    mut commands: Commands,
    shape_query: Query<(Entity, &NetworkId)>
) {
    for event in delete_shape_command_reader.iter() {
        for (entity, network_id) in shape_query.iter() {
            if *network_id == event.0 {
                commands.entity(entity).despawn();
                dbg!("Deleting shape with entity ID", entity);
            }
        }
    }
}