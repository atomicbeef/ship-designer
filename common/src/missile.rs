use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use packets_derive::{IntoPacket, TryFromPacket};

use crate::compact_transform::CompactTransform;
use crate::fixed_update::AddFixedEvent;
use crate::network_id::NetworkId;

#[derive(Component)]
pub struct Missile {
    pub power: f32,
}

impl Missile {
    pub fn new(power: f32) -> Self {
        Self { power }
    }
}

#[derive(Bundle)]
pub struct MissileBundle {
    pub missile: Missile,
    pub network_id: NetworkId,
    #[bundle]
    pub transform: TransformBundle,
    pub collider: Collider,
    pub sensor: Sensor,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub ccd: Ccd,
}

impl Default for MissileBundle {
    fn default() -> Self {
        Self {
            missile: Missile { power: 1.0 },
            network_id: NetworkId::from(0),
            transform: TransformBundle::default(),
            collider: Collider::default(),
            sensor: Sensor,
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            ccd: Ccd::enabled(),
        }
    }
}

#[derive(Debug, IntoPacket, TryFromPacket)]
#[PacketType(SpawnMissile)]
pub struct SpawnMissileRequest {
    pub transform: CompactTransform,
    pub velocity: Vec3
}

#[derive(Debug, IntoPacket, TryFromPacket)]
#[PacketType(SpawnMissile)]
pub struct SpawnMissileCommand {
    pub transform: CompactTransform,
    pub velocity: Vec3,
    pub network_id: NetworkId,
}

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(ExplodeMissile)]
pub struct ExplodeMissileCommand {
    pub network_id: NetworkId,
    pub transform: CompactTransform,
}

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        let mut packet = packets::Packet::new(packets::PacketType::SpawnMissile);
        <u8 as packets::PacketSerialize>::serialize(&10, &mut packet);

        app.add_fixed_event::<SpawnMissileRequest>();
        app.add_fixed_event::<SpawnMissileCommand>();
        app.add_fixed_event::<ExplodeMissileCommand>();
    }
}