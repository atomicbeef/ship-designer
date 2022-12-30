use bevy::prelude::Transform;
use bevy::reflect::Reflect;

use crate::packets::{Packet, PacketError, PacketSerialize, PacketDeserialize};

// Position relative to physics body
#[derive(Clone, Copy, Debug, Reflect)]
pub struct ShapePos {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

impl ShapePos {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

impl PacketSerialize for ShapePos {
    fn serialize(&self, packet: &mut Packet) {
        self.x.serialize(packet);
        self.y.serialize(packet);
        self.z.serialize(packet);
    }
}

impl PacketDeserialize for ShapePos {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let x = i16::deserialize(packet)?;
        let y = i16::deserialize(packet)?;
        let z = i16::deserialize(packet)?;

        Ok(Self { x, y, z })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShapeTransform {
    pub translation: ShapePos,
    // TODO: Add rotation
}

impl ShapeTransform {
    pub fn new(translation: ShapePos) -> Self {
        ShapeTransform { translation }
    }

    pub fn from_xyz(x: i16, y: i16, z: i16) -> Self {
        ShapeTransform { translation: ShapePos::new(x, y, z) }
    }
}

impl From<ShapeTransform> for Transform {
    fn from(value: ShapeTransform) -> Self {
        Self::from_xyz(value.translation.x.into(), value.translation.y.into(), value.translation.z.into())
    }
}

impl PacketSerialize for ShapeTransform {
    fn serialize(&self, packet: &mut Packet) {
        self.translation.serialize(packet);
    }
}

impl PacketDeserialize for ShapeTransform {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let translation = ShapePos::deserialize(packet)?;
        Ok(Self{ translation })
    }
}