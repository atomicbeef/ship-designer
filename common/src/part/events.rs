use crate::network_id::NetworkId;
use packets_derive::{IntoPacket, TryFromPacket};
use crate::part::{Material, PartId};
use crate::compact_transform::CompactTransform;

#[derive(Debug, IntoPacket, TryFromPacket)]
#[PacketType(PlacePart)]
pub struct PlacePartRequest {
    pub part_id: PartId,
    pub part_transform: CompactTransform,
    pub construct_network_id: NetworkId
}

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(PlacePart)]
pub struct PlacePartCommand {
    pub part_id: PartId,
    pub transform: CompactTransform,
    pub part_network_id: NetworkId,
    pub construct_network_id: NetworkId
}

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(DeletePart)]
pub struct DeletePartRequest(pub NetworkId);

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(DeletePart)]
pub struct DeletePartCommand(pub NetworkId);

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(VoxelUpdate)]
pub struct VoxelUpdate {
    pub network_id: NetworkId,
    pub voxels: Vec<Material>
}