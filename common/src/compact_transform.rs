use bevy::prelude::{Transform, Vec3, Quat};

use crate::packets::{Packet, PacketError, PacketSerialize, PacketDeserialize};

#[derive(Clone, Copy, Debug)]
pub struct CompactTransform {
    pub translation: Vec3,
    pub rotation: Quat
}

impl CompactTransform {
    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        CompactTransform { translation, rotation }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        CompactTransform { translation: Vec3::new(x, y, z), rotation: Quat::IDENTITY }
    }
}

impl From<CompactTransform> for Transform {
    fn from(value: CompactTransform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation,
            scale: Vec3::new(1.0, 1.0, 1.0)
        }
    }
}

impl PacketSerialize for CompactTransform {
    fn serialize(&self, packet: &mut Packet) {
        self.translation.serialize(packet);
        self.rotation.serialize(packet);
    }
}

impl PacketDeserialize for CompactTransform {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let translation = Vec3::deserialize(packet)?;
        let rotation = Quat::deserialize(packet)?;

        Ok(Self::new(translation, rotation))
    }
}

impl From<Transform> for CompactTransform {
    fn from(value: Transform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation
        }
    }
}