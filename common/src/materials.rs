use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Material {
    Empty,
    Aluminum
}

impl PacketSerialize for Material {
    fn serialize(&self, packet: &mut Packet) {
        u8::from(*self).serialize(packet);
    }
}

impl PacketDeserialize for Material {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let material_byte = u8::deserialize(packet)?;

        match Material::try_from(material_byte) {
            Ok(material) => Ok(material),
            Err(_) => Err(PacketError::InvalidPacketError(packet.clone()))
        }
    }
}