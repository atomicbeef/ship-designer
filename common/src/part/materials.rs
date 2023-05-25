use bevy::utils::HashMap;
use bevy::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
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

#[derive(Resource)]
pub struct MaterialResistances(HashMap<Material, f32>);

impl MaterialResistances {
    pub fn new() -> Self {
        let mut resistances = HashMap::new();
        resistances.insert(Material::Empty, 0.0);
        resistances.insert(Material::Aluminum, 0.0);

        Self(resistances)
    }

    pub fn get(&self, material: Material) -> f32 {
        self.0.get(&material).copied().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use packets::{Packet, PacketSerialize, PacketDeserialize, PacketType};

    use crate::part::materials::Material;

    #[test]
    fn material_serialize_deserialize() {
        let mut packet = Packet::new(PacketType::VoxelUpdate);

        let x = Material::Aluminum;
        x.serialize(&mut packet);

        let y = Material::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }
}