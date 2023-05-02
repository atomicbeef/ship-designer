use crate::network_id::NetworkId;
use packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::part::{Material, PartId};
use crate::compact_transform::CompactTransform;

#[derive(Debug)]
pub struct PlacePartRequest {
    pub part_id: PartId,
    pub part_transform: CompactTransform,
    pub construct_network_id: NetworkId
}

impl TryFrom<Packet> for PlacePartRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let part_id = PartId::deserialize(&mut packet)?;
        let part_transform = CompactTransform::deserialize(&mut packet)?;
        let construct_network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self { part_id, part_transform, construct_network_id })
    }
}

impl From<&PlacePartRequest> for Packet {
    fn from(place_part_request: &PlacePartRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlacePart);
        place_part_request.part_id.serialize(&mut packet);
        place_part_request.part_transform.serialize(&mut packet);
        place_part_request.construct_network_id.serialize(&mut packet);
        packet
    }
}

pub struct PlacePartCommand {
    pub part_id: PartId,
    pub transform: CompactTransform,
    pub part_network_id: NetworkId,
    pub construct_network_id: NetworkId
}

impl TryFrom<Packet> for PlacePartCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let part_id = PartId::deserialize(&mut packet)?;
        let transform = CompactTransform::deserialize(&mut packet)?;
        let part_network_id = NetworkId::deserialize(&mut packet)?;
        let construct_network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self { part_id, transform, part_network_id, construct_network_id })
    }
}

impl From<&PlacePartCommand> for Packet {
    fn from(place_part_command: &PlacePartCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlacePart);
        place_part_command.part_id.serialize(&mut packet);
        place_part_command.transform.serialize(&mut packet);
        place_part_command.part_network_id.serialize(&mut packet);
        place_part_command.construct_network_id.serialize(&mut packet);
        packet
    }
}

pub struct DeletePartRequest(pub NetworkId);

impl TryFrom<Packet> for DeletePartRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self(network_id))
    }
}

impl From<&DeletePartRequest> for Packet {
    fn from(delete_part_request: &DeletePartRequest) -> Self {
        let mut packet = Packet::new(PacketType::DeletePart);
        delete_part_request.0.serialize(&mut packet);
        packet
    }
}

pub struct DeletePartCommand(pub NetworkId);

impl TryFrom<Packet> for DeletePartCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = NetworkId::deserialize(&mut packet)?;
        Ok(Self(id))
    }
}

impl From<&DeletePartCommand> for Packet {
    fn from(delete_part_command: &DeletePartCommand) -> Self {
        let mut packet = Packet::new(PacketType::DeletePart);
        delete_part_command.0.serialize(&mut packet);
        packet
    }
}

pub struct VoxelUpdate {
    pub network_id: NetworkId,
    pub voxels: Vec<Material>
}

impl TryFrom<Packet> for VoxelUpdate {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id = NetworkId::deserialize(&mut packet)?;
        let voxels: Vec<Material> = Vec::deserialize(&mut packet)?;

        Ok(Self { network_id, voxels })
    }
}

impl From<&VoxelUpdate> for Packet {
    fn from(value: &VoxelUpdate) -> Self {
        let mut packet = Packet::new(PacketType::VoxelUpdate);

        value.network_id.serialize(&mut packet);
        value.voxels.serialize(&mut packet);

        packet
    }
}