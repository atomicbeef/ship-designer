use bevy::prelude::*;
use common::player::Players;
use uflow::SendMode;

use common::channels::Channel;
use common::events::building::{PlaceShapeRequest, PlaceShapeCommand, DeleteShapeRequest, DeleteShapeCommand};
use common::network_id::{NetworkId, entity_from_network_id};
use common::packets::Packet;
use common::shape::{Shapes, ShapeHandle};

use crate::network_id_generator::NetworkIdGenerator;
use crate::server_state::ServerState;

pub fn spawn_shape(
    commands: &mut Commands,
    shape_handle: ShapeHandle,
    transform: Transform,
    shape_network_id: NetworkId,
    body: Entity
) -> Entity {
    let shape_entity = commands.spawn_empty()
        .insert(shape_handle)
        .insert(shape_network_id)
        .insert(transform)
        .id();
    
    commands.entity(body).add_child(shape_entity);

    shape_entity
}

pub fn confirm_place_shape_requests(
    mut place_shape_request_reader: EventReader<PlaceShapeRequest>,
    mut send_place_shape_writer: EventWriter<PlaceShapeCommand>,
    mut commands: Commands,
    mut network_id_generator: ResMut<NetworkIdGenerator>,
    shapes: Res<Shapes>,
    body_query: Query<(Entity, &NetworkId)>
) {
    for place_shape_request in place_shape_request_reader.iter() {
        // TODO: Prevent shapes from being placed inside of each other

        // Spawn shape
        let network_id = network_id_generator.generate();
        let shape_handle = shapes.get_handle(place_shape_request.shape_id);
        let body = entity_from_network_id(body_query.iter(), place_shape_request.body_network_id).unwrap();

        spawn_shape(
            &mut commands,
            shape_handle,
            Transform::from(place_shape_request.shape_transform),
            network_id,
            body
        );

        send_place_shape_writer.send(PlaceShapeCommand {
            shape_id: place_shape_request.shape_id,
            shape_network_id: network_id,
            transform: place_shape_request.shape_transform,
            body_network_id: place_shape_request.body_network_id
        });
    }
}

pub fn confirm_delete_shape_requests(
    mut commands: Commands,
    mut delete_shape_request_reader: EventReader<DeleteShapeRequest>,
    mut send_delete_shape_writer: EventWriter<DeleteShapeCommand>,
    network_id_query: Query<(Entity, &NetworkId), With<ShapeHandle>>,
    parent_query: Query<&Parent>
) {
    for delete_shape_request in delete_shape_request_reader.iter() {
        for (entity, network_id) in network_id_query.iter() {
            if *network_id == delete_shape_request.0 {
                let ship = parent_query.get(entity).unwrap().get();
                commands.entity(ship).remove_children(&[entity]);
                commands.entity(entity).despawn();
                send_delete_shape_writer.send(DeleteShapeCommand(delete_shape_request.0));
                break;
            }
        }
    }
}

pub fn send_place_shape_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_place_shape_reader: EventReader<PlaceShapeCommand>
) {
    for place_shape_command in send_place_shape_reader.iter() {
        let packet = Packet::from(place_shape_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}

pub fn send_delete_shape_commands(
    mut server_state: NonSendMut<ServerState>,
    players: Res<Players>,
    mut send_delete_shape_reader: EventReader<DeleteShapeCommand>
) {
    for delete_shape_command in send_delete_shape_reader.iter() {
        let packet = Packet::from(delete_shape_command);

        for player_id in players.ids() {
            server_state.send_to_player(
                *player_id,
                (&packet).into(),
                Channel::ShapeCommands.into(),
                SendMode::Reliable
            );
        }
    }
}