use bevy::input::mouse::MouseButton;
use bevy_mod_picking::{PickableBundle, PickingRaycastSet, RayCastSource};
use bevy::prelude::*;
use uflow::SendMode;

use common::channels::Channel;
use common::events::{PlaceBlockRequest, PlaceBlockCommand, DeleteBlockRequest, DeleteBlockCommand};
use common::grid::{Grid, GridPos};
use common::packets::Packet;

use crate::connection_state::ConnectionState;

pub fn build_request_events(
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut place_block_request_writer: EventWriter<PlaceBlockRequest>,
    mut delete_block_request_writer: EventWriter<DeleteBlockRequest>,
    intersection_query: Query<&RayCastSource<PickingRaycastSet>>,
    transform_query: Query<&Transform>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let intersection_data = intersection_query.iter().next().unwrap().intersect_top();
        if let Some(data) = intersection_data {
            // Block deletion
            if keys.pressed(KeyCode::LAlt) {
                if let Ok(block_transform) = transform_query.get(data.0) {
                    let block_pos: GridPos = block_transform.into();

                    delete_block_request_writer.send(DeleteBlockRequest(block_pos));
                }
            // Block placement
            } else {
                if let Ok(origin_block_transform) = transform_query.get(data.0) {
                    let block_pos = (origin_block_transform.translation + data.1.normal()).into();

                    place_block_request_writer.send(PlaceBlockRequest(block_pos));
                }
            }
        }
    }
}

pub fn send_place_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut place_block_request_reader: EventReader<PlaceBlockRequest>
) {
    for place_block_request in place_block_request_reader.iter() {
        let packet: Packet = place_block_request.into();
        connection_state.server.send(packet.into(), Channel::BlockCommands.into(), SendMode::Reliable);
    }
}

pub fn send_delete_block_requests(
    mut connection_state: ResMut<ConnectionState>,
    mut delete_block_request_reader: EventReader<DeleteBlockRequest>
) {
    for delete_block_request in delete_block_request_reader.iter() {
        let packet: Packet = delete_block_request.into();
        connection_state.server.send(packet.into(), Channel::BlockCommands.into(), SendMode::Reliable);
    }
}

pub fn place_blocks(
    mut place_block_command_reader: EventReader<PlaceBlockCommand>,
    mut grid_query: Query<&mut Grid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for event in place_block_command_reader.iter() {
        let block_id = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform { translation: Vec3::new(event.0.x as f32, event.0.y as f32, event.0.z as f32), ..Default::default() },
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default())
        .id();

        if let Some(mut grid) = grid_query.iter_mut().next() {
            grid.set(&event.0, Some(block_id));
        }
    }
}

pub fn delete_blocks(
    mut delete_block_command_reader: EventReader<DeleteBlockCommand>,
    mut commands: Commands,
    mut grid_query: Query<&mut Grid>
) {
    for event in delete_block_command_reader.iter() {
        if let Some(mut grid) = grid_query.iter_mut().next() {
            match grid.get(&event.0) {
                Some(entity) => {
                    commands.entity(entity).despawn();
                    grid.set(&event.0, None);
                },
                None => { warn!("No entity to delete at {:?}!", event.0) }
            };
        }
    }
}