use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::compact_transform::CompactTransform;
use crate::network_id::NetworkId;
use packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};

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
    pub gravity_scale: GravityScale,
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
            gravity_scale: GravityScale(0.0),
        }
    }
}

#[derive(Debug)]
pub struct SpawnMissileRequest {
    pub transform: CompactTransform,
    pub velocity: Vec3
}

impl TryFrom<Packet> for SpawnMissileRequest {
    type Error = PacketError;
    
    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let transform = CompactTransform::deserialize(&mut packet)?;
        let velocity = Vec3::deserialize(&mut packet)?;

        Ok(Self { transform, velocity })
    }
}

impl From<&SpawnMissileRequest> for Packet {
    fn from(value: &SpawnMissileRequest) -> Self {
        let mut packet = Packet::new(PacketType::SpawnMissile);

        value.transform.serialize(&mut packet);
        value.velocity.serialize(&mut packet);

        packet
    }
}

#[derive(Debug)]
pub struct SpawnMissileCommand {
    pub transform: CompactTransform,
    pub velocity: Vec3,
    pub network_id: NetworkId,
}

impl TryFrom<Packet> for SpawnMissileCommand {
    type Error = PacketError;
    
    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let transform = CompactTransform::deserialize(&mut packet)?;
        let velocity = Vec3::deserialize(&mut packet)?;
        let network_id = NetworkId::deserialize(&mut packet)?;

        Ok(Self { transform, velocity, network_id })
    }
}

impl From<&SpawnMissileCommand> for Packet {
    fn from(value: &SpawnMissileCommand) -> Self {
        let mut packet = Packet::new(PacketType::SpawnMissile);

        value.transform.serialize(&mut packet);
        value.velocity.serialize(&mut packet);
        value.network_id.serialize(&mut packet);

        packet
    }
}

pub struct ExplodeMissileCommand {
    pub network_id: NetworkId,
    pub transform: CompactTransform,
}

impl TryFrom<Packet> for ExplodeMissileCommand {
    type Error = PacketError;
    
    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id = NetworkId::deserialize(&mut packet)?;
        let transform = CompactTransform::deserialize(&mut packet)?;

        Ok(Self { network_id, transform })
    }
}

impl From<&ExplodeMissileCommand> for Packet {
    fn from(value: &ExplodeMissileCommand) -> Self {
        let mut packet = Packet::new(PacketType::ExplodeMissile);

        value.network_id.serialize(&mut packet);
        value.transform.serialize(&mut packet);

        packet
    }
}

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMissileRequest>();
        app.add_event::<SpawnMissileCommand>();
        app.add_event::<ExplodeMissileCommand>();
    }
}