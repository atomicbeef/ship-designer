use crate::grid::GridPos;
use crate::network_id::NetworkId;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::player::Player;
use crate::shape::{ShapeHandle, ShapeHandleId};

pub struct PlaceShapeRequest(pub ShapeHandleId, pub GridPos);

impl TryFrom<Packet> for PlaceShapeRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_handle_id: ShapeHandleId = packet.read()?;
        let pos: GridPos = packet.read()?;
        Ok(Self(shape_handle_id, pos))
    }
}

impl From<&PlaceShapeRequest> for Packet {
    fn from(place_shape_request: &PlaceShapeRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        packet.write(place_shape_request.0);
        packet.write(place_shape_request.1);
        packet
    }
}

pub struct PlaceShapeCommand {
    pub shape_handle_id: ShapeHandleId,
    pub pos: GridPos,
    pub network_id: NetworkId
}

impl TryFrom<Packet> for PlaceShapeCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let shape_id: ShapeHandleId = packet.read()?;
        let pos: GridPos = packet.read()?;
        let network_id: NetworkId = packet.read()?;
        Ok(Self { shape_handle_id: shape_id, pos, network_id })
    }
}

impl From<&PlaceShapeCommand> for Packet {
    fn from(place_shape_command: &PlaceShapeCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlaceShape);
        packet.write(place_shape_command.shape_handle_id);
        packet.write(place_shape_command.pos);
        packet.write(place_shape_command.network_id);
        packet
    }
}

pub struct DeleteShapeRequest(pub NetworkId);

impl TryFrom<Packet> for DeleteShapeRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let network_id: NetworkId = packet.read()?;
        Ok(Self(network_id))
    }
}

impl From<&DeleteShapeRequest> for Packet {
    fn from(delete_block_request: &DeleteShapeRequest) -> Self {
        let mut packet = Packet::new(PacketType::DeleteShape);
        packet.write(delete_block_request.0);
        packet
    }
}

pub struct DeleteShapeCommand(pub NetworkId);

impl TryFrom<Packet> for DeleteShapeCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id: NetworkId = packet.read()?;
        Ok(Self(id))
    }
}

impl From<&DeleteShapeCommand> for Packet {
    fn from(delete_shape_command: &DeleteShapeCommand) -> Self {
        let mut packet = Packet::new(PacketType::DeleteShape);
        packet.write(delete_shape_command.0);
        packet
    }
}

pub struct PlayerConnected {
    pub id: u8,
    pub name: String
}

impl TryFrom<Packet> for PlayerConnected {
    type Error = PacketError;
    
    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = packet.read()?;
        let name: String = packet.read()?;
        Ok(Self { id, name })
    }
}

impl From<&PlayerConnected> for Packet {
    fn from(player_connected: &PlayerConnected) -> Self {
        let mut packet = Packet::new(PacketType::PlayerConnected);
        packet.write(player_connected.id);
        packet.write(&player_connected.name);
        packet
    }
}

pub struct PlayerDisconnected(pub u8);

impl TryFrom<Packet> for PlayerDisconnected {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = packet.read()?;
        Ok(Self(id))
    }
}

impl From<&PlayerDisconnected> for Packet {
    fn from(player_disconnected: &PlayerDisconnected) -> Self {
        let mut packet = Packet::new(PacketType::PlayerDisconnected);
        packet.write(player_disconnected.0);
        packet
    }
}

pub struct InitialState {
    pub players: Vec<Player>,
    pub shapes: Vec<(ShapeHandle, GridPos, NetworkId)>,
}

impl TryFrom<Packet> for InitialState {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let players: Vec<Player> = packet.read()?;
        let shapes: Vec<(ShapeHandle, GridPos, NetworkId)> = packet.read()?;
        Ok(Self { players, shapes })
    }
}

impl From<&InitialState> for Packet {
    fn from(initial_state: &InitialState) -> Self {
        let mut packet = Packet::new(PacketType::InitialState);
        packet.write(&initial_state.players);
        packet.write(&initial_state.shapes);
        packet
    }
}