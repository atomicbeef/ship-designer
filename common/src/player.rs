use bevy::prelude::*;
use bevy::transform::TransformBundle;
use bevy_rapier3d::prelude::*;

use packets_derive::{PacketSerialize, PacketDeserialize};

#[derive(Clone, Copy, Debug, Component, PartialEq, Eq, Hash, PacketSerialize, PacketDeserialize, Reflect)]
pub struct PlayerId {
    id: u8
}

impl From<u8> for PlayerId {
    fn from(id: u8) -> Self {
        PlayerId { id }
    }
}

#[derive(Clone, Debug, Component, PacketSerialize, PacketDeserialize, Reflect)]
pub struct PlayerName {
    name: String
}

impl From<String> for PlayerName {
    fn from(name: String) -> Self {
        PlayerName { name }
    }
}

impl std::fmt::Display for PlayerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub name: PlayerName,
    pub transform: TransformBundle,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub gravity_scale: GravityScale,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            id: PlayerId::from(0),
            name: PlayerName::from("".to_string()),
            transform: TransformBundle::default(),
            collider: Collider::capsule(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0.5),
            rigid_body: RigidBody::Dynamic,
            gravity_scale: GravityScale(0.0),
        }
    }
}

#[derive(Component)]
pub struct LocalPlayer;