use crate::network_id::NetworkId;
use crate::materials::Material;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};

pub struct UpdateVoxels {
    pub network_id: NetworkId,
    pub voxels: Vec<Material>
}

impl TryFrom<Packet> for UpdateVoxels {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id = NetworkId::deserialize(&mut packet)?;
        let voxels: Vec<Material> = Vec::deserialize(&mut packet)?;

        Ok(Self { network_id, voxels })
    }
}

impl From<&UpdateVoxels> for Packet {
    fn from(value: &UpdateVoxels) -> Self {
        let mut packet = Packet::new(PacketType::UpdateVoxels);

        value.network_id.serialize(&mut packet);
        value.voxels.serialize(&mut packet);

        packet
    }
}