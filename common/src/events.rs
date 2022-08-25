use crate::grid::GridPos;
use crate::packets::{read_grid_pos, write_grid_pos, Packet, PacketError, PacketType};

pub struct PlaceBlockRequest(pub GridPos);

impl TryFrom<Packet> for PlaceBlockRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos = read_grid_pos(&mut packet)?;
        Ok(Self(pos))
    }
}

impl From<&PlaceBlockRequest> for Packet {
    fn from(place_block_request: &PlaceBlockRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlaceBlock);
        write_grid_pos(&mut packet, place_block_request.0);
        packet
    }
}

pub struct PlaceBlockCommand(pub GridPos);

impl TryFrom<Packet> for PlaceBlockCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos = read_grid_pos(&mut packet)?;
        Ok(Self(pos))
    }
}

impl From<&PlaceBlockCommand> for Packet {
    fn from(place_block_command: &PlaceBlockCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlaceBlock);
        write_grid_pos(&mut packet, place_block_command.0);
        packet
    }
}

pub struct DeleteBlockRequest(pub GridPos);

impl TryFrom<Packet> for DeleteBlockRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos = read_grid_pos(&mut packet)?;
        Ok(Self(pos))
    }
}

impl From<&DeleteBlockRequest> for Packet {
    fn from(delete_block_request: &DeleteBlockRequest) -> Self {
        let mut packet = Packet::new(PacketType::DeleteBlock);
        write_grid_pos(&mut packet, delete_block_request.0);
        packet
    }
}

pub struct DeleteBlockCommand(pub GridPos);

impl TryFrom<Packet> for DeleteBlockCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos = read_grid_pos(&mut packet)?;
        Ok(Self(pos))
    }
}

impl From<&DeleteBlockCommand> for Packet {
    fn from(delete_block_command: &DeleteBlockCommand) -> Self {
        let mut packet = Packet::new(PacketType::DeleteBlock);
        write_grid_pos(&mut packet, delete_block_command.0);
        packet
    }
}