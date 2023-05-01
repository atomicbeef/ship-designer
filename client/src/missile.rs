use bevy::prelude::*;
use bevy::prelude::shape::Cube;
use bevy_rapier3d::prelude::*;
use common::entity_lookup::lookup;
use common::network_id::NetworkId;
use uflow::SendMode;

use common::missile::{SpawnMissileRequest, SpawnMissileCommand, MissileBundle, ExplodeMissileCommand, Missile};
use common::packets::Packet; 
use common::channels::Channel;

use crate::connection_state::ConnectionState;

fn request_spawn_missiles(
    keys: Res<Input<KeyCode>>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut spawn_event_writer: EventWriter<SpawnMissileRequest>,
) {
    if keys.just_pressed(KeyCode::M) {
        let camera_transform = match camera_query.iter().next() {
            Some(camera_transform) => camera_transform,
            None => { return; },
        };

        spawn_event_writer.send(SpawnMissileRequest {
            transform: (*camera_transform).into(),
            velocity: camera_transform.forward() * 100.0,
        });
    }
}

fn send_spawn_missile_requests(
    mut spawn_event_reader: EventReader<SpawnMissileRequest>,
    mut connection_state: ResMut<ConnectionState>,
) {
    for spawn_event in spawn_event_reader.iter() {
        let packet = Packet::from(spawn_event);

        connection_state.client.send(
            (&packet).into(),
            Channel::Missile.into(),
            SendMode::Reliable
        );
    }
}

fn spawn_missiles(
    mut spawn_event_reader: EventReader<SpawnMissileCommand>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for spawn_event in spawn_event_reader.iter() {
        commands.spawn(
            MissileBundle {
                missile: Missile::new(5.0),
                network_id: spawn_event.network_id,
                transform: TransformBundle::from_transform(spawn_event.transform.into()),
                velocity: Velocity::linear(spawn_event.velocity),
                collider: Collider::cuboid(0.25, 0.25, 0.25),
                ..Default::default()
            })
            .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(Cube::new(0.5))),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from(spawn_event.transform),
                ..Default::default()
            });
    }
}

fn explode_missiles(
    mut explode_event_reader: EventReader<ExplodeMissileCommand>,
    network_id_query: Query<(Entity, &NetworkId), With<Missile>>,
    mut commands: Commands
) {
    for explode_event in explode_event_reader.iter() {
        if let Some(entity) = lookup(&network_id_query, &explode_event.network_id) {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ClientMissilePlugin;

impl Plugin for ClientMissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_missiles)
            .add_system(request_spawn_missiles)
            .add_system(send_spawn_missile_requests)
            .add_system(explode_missiles);
    }
}