use bevy::prelude::{Transform, Vec3, Quat};

use super::{Packet, PacketSerialize, PacketDeserialize, PacketError};

impl PacketSerialize for Vec3 {
    fn serialize(&self, packet: &mut Packet) {
        self.to_array().serialize(packet);
    }
}

impl PacketDeserialize for Vec3 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let vector: Vec<f32> = Vec::deserialize(packet)?;
        Ok(Vec3::from_slice(&vector))
    }
}

impl PacketSerialize for Quat {
    fn serialize(&self, packet: &mut Packet) {
        self.to_array().serialize(packet);
    }
}

impl PacketDeserialize for Quat {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let quat: Vec<f32> = Vec::deserialize(packet)?;
        Ok(Quat::from_slice(&quat))
    }
}

impl PacketSerialize for Transform {
    fn serialize(&self, packet: &mut Packet) {
        self.translation.serialize(packet);
        self.rotation.serialize(packet);
        self.scale.serialize(packet);
    }
}

impl PacketDeserialize for Transform {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let translation = Vec3::deserialize(packet)?;
        let rotation = Quat::deserialize(packet)?;
        let scale = Vec3::deserialize(packet)?;
        Ok(Transform { translation, rotation, scale })
    }
}