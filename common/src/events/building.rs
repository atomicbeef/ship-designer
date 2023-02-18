use crate::network_id::NetworkId;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::shape::ShapeId;
use crate::compact_transform::CompactTransform;

#[derive(Debug)]
pub struct PlaceShapeRequest {
    pub shape_id: ShapeId,
    pub shape_transform: CompactTransform,
    pub body_network_id: NetworkId
}

impl TryFrom<Packet> for PlaceShapeRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_id = ShapeId::deserialize(&mut packet)?;
        let shape_transform = CompactTransform::deserialize(&mut packet)?;
        let body_network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self { shape_id, shape_transform, body_network_id })
    }
}

impl From<&PlaceShapeRequest> for Packet {
    fn from(place_shape_request: &PlaceShapeRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        place_shape_request.shape_id.serialize(&mut packet);
        place_shape_request.shape_transform.serialize(&mut packet);
        place_shape_request.body_network_id.serialize(&mut packet);
        packet
    }
}

pub struct PlaceShapeCommand {
    pub shape_id: ShapeId,
    pub transform: CompactTransform,
    pub shape_network_id: NetworkId,
    pub body_network_id: NetworkId
}

impl TryFrom<Packet> for PlaceShapeCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_id = ShapeId::deserialize(&mut packet)?;
        let transform = CompactTransform::deserialize(&mut packet)?;
        let shape_network_id = NetworkId::deserialize(&mut packet)?;
        let body_network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self { shape_id, transform, shape_network_id, body_network_id })
    }
}

impl From<&PlaceShapeCommand> for Packet {
    fn from(place_shape_command: &PlaceShapeCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        place_shape_command.shape_id.serialize(&mut packet);
        place_shape_command.transform.serialize(&mut packet);
        place_shape_command.shape_network_id.serialize(&mut packet);
        place_shape_command.body_network_id.serialize(&mut packet);
        packet
    }
}

pub struct DeleteShapeRequest(pub NetworkId);

impl TryFrom<Packet> for DeleteShapeRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self(network_id))
    }
}

impl From<&DeleteShapeRequest> for Packet {
    fn from(delete_block_request: &DeleteShapeRequest) -> Self {
        let mut packet = Packet::new(PacketType::DeleteShape);
        delete_block_request.0.serialize(&mut packet);
        packet
    }
}

pub struct DeleteShapeCommand(pub NetworkId);

impl TryFrom<Packet> for DeleteShapeCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = NetworkId::deserialize(&mut packet)?;
        Ok(Self(id))
    }
}

impl From<&DeleteShapeCommand> for Packet {
    fn from(delete_shape_command: &DeleteShapeCommand) -> Self {
        let mut packet = Packet::new(PacketType::DeleteShape);
        delete_shape_command.0.serialize(&mut packet);
        packet
    }
}