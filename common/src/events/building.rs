use crate::network_id::NetworkId;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::shape::ShapeId;
use crate::shape_transform::ShapeTransform;

pub struct PlaceShapeRequest(pub ShapeId, pub ShapeTransform);

impl TryFrom<Packet> for PlaceShapeRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_id = ShapeId::deserialize(&mut packet)?;
        let transform = ShapeTransform::deserialize(&mut packet)?;
        Ok(Self(shape_id, transform))
    }
}

impl From<&PlaceShapeRequest> for Packet {
    fn from(place_shape_request: &PlaceShapeRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        place_shape_request.0.serialize(&mut packet);
        place_shape_request.1.serialize(&mut packet);
        packet
    }
}

pub struct PlaceShapeCommand {
    pub shape_id: ShapeId,
    pub transform: ShapeTransform,
    pub network_id: NetworkId
}

impl TryFrom<Packet> for PlaceShapeCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_id = ShapeId::deserialize(&mut packet)?;
        let transform = ShapeTransform::deserialize(&mut packet)?;
        let network_id = NetworkId::deserialize(&mut packet)?;
        Ok(Self { shape_id, transform, network_id })
    }
}

impl From<&PlaceShapeCommand> for Packet {
    fn from(place_shape_command: &PlaceShapeCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        place_shape_command.shape_id.serialize(&mut packet);
        place_shape_command.transform.serialize(&mut packet);
        place_shape_command.network_id.serialize(&mut packet);
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