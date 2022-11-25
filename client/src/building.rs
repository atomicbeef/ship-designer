use bevy::input::mouse::MouseButton;
use bevy_mod_picking::{PickableBundle, PickingRaycastSet, RaycastSource};
use bevy::prelude::*;
use common::network_id::NetworkId;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::packets::Packet;
use common::shape::{ShapeHandle, Shapes, ShapeHandleId, ShapeHandleType};

use crate::connection_state::ConnectionState;
use crate::mesh_generation::generate_shape_mesh;

pub fn build_request_events(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut place_shape_request_writer: EventWriter<PlaceShapeRequest>,
    mut delete_shape_request_writer: EventWriter<DeleteShapeRequest>,
    intersection_query: Query<&RaycastSource<PickingRaycastSet>>,
    transform_query: Query<&Transform>,
    network_id_query: Query<&NetworkId>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let intersection_data = intersection_query.iter().next().unwrap().get_nearest_intersection();
        if let Some(data) = intersection_data {
            // Block deletion
            if keys.pressed(KeyCode::LAlt) {
                let network_id = network_id_query.get(data.0).unwrap();
                delete_shape_request_writer.send(DeleteShapeRequest(*network_id));
            // Block placement
            } else {
                if let Ok(origin_shape_transform) = transform_query.get(data.0) {
                    let shape_handle_id = ShapeHandleId::from(0);
                    let block_pos = (origin_shape_transform.translation + data.1.normal()).into();

                    place_shape_request_writer.send(PlaceShapeRequest(shape_handle_id, block_pos));
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
        connection_state.server.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn send_delete_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_block_request_reader: EventReader<DeleteShapeRequest>
) {
    for delete_block_request in delete_block_request_reader.iter() {
        let packet: Packet = delete_block_request.into();
        connection_state.server.send((&packet).into(), Channel::ShapeCommands.into(), SendMode::Reliable);
    }
}

pub fn spawn_shape(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    shapes: &Shapes,
    shape_handle: ShapeHandle,
    transform: Transform,
    network_id: NetworkId
) -> Entity {
    let shape = shapes.get(&shape_handle).unwrap();
    let mesh = generate_shape_mesh(shape);

    commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: transform,
            ..Default::default()
        })
        .insert(shape_handle)
        .insert(network_id)
        .insert(PickableBundle::default())
        .id()
}

pub fn place_shapes(
    mut place_shape_command_reader: EventReader<PlaceShapeCommand>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    shapes: Res<Shapes>
) {
    for event in place_shape_command_reader.iter() {
        let transform = Transform::from(event.pos);
        let entity = spawn_shape(
            &mut commands,
            &mut meshes,
            &mut materials,
            &shapes,
            ShapeHandle::new(ShapeHandleId::from(0), ShapeHandleType::ReadOnly),
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