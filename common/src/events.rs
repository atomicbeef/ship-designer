use crate::grid::GridPos;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::player::Player;

pub struct PlaceBlockRequest(pub GridPos);

impl TryFrom<Packet> for PlaceBlockRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos: GridPos = packet.read()?;
        Ok(Self(pos))
    }
}

impl From<&PlaceBlockRequest> for Packet {
    fn from(place_block_request: &PlaceBlockRequest) -> Self {
        let mut packet = Packet::new(PacketType::PlaceBlock);
        packet.write(place_block_request.0);
        packet
    }
}

pub struct PlaceBlockCommand(pub GridPos);

impl TryFrom<Packet> for PlaceBlockCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos: GridPos = packet.read()?;
        Ok(Self(pos))
    }
}

impl From<&PlaceBlockCommand> for Packet {
    fn from(place_block_command: &PlaceBlockCommand) -> Self {
        let mut packet = Packet::new(PacketType::PlaceBlock);
        packet.write(place_block_command.0);
        packet
    }
}

pub struct DeleteBlockRequest(pub GridPos);

impl TryFrom<Packet> for DeleteBlockRequest {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos: GridPos = packet.read()?;
        Ok(Self(pos))
    }
}

impl From<&DeleteBlockRequest> for Packet {
    fn from(delete_block_request: &DeleteBlockRequest) -> Self {
        let mut packet = Packet::new(PacketType::DeleteBlock);
        packet.write(delete_block_request.0);
        packet
    }
}

pub struct DeleteBlockCommand(pub GridPos);

impl TryFrom<Packet> for DeleteBlockCommand {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let pos: GridPos = packet.read()?;
        Ok(Self(pos))
    }
}

impl From<&DeleteBlockCommand> for Packet {
    fn from(delete_block_command: &DeleteBlockCommand) -> Self {
        let mut packet = Packet::new(PacketType::DeleteBlock);
        packet.write(delete_block_command.0);
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
    pub grid_positions: Vec<GridPos>,
}

impl TryFrom<Packet> for InitialState {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let players: Vec<Player> = packet.read()?;
        let grid_positions: Vec<GridPos> = packet.read()?;
        Ok(Self { players, grid_positions })
    }
}

impl From<&InitialState> for Packet {
    fn from(initial_state: &InitialState) -> Self {
        let mut packet = Packet::new(PacketType::InitialState);
        packet.write(&initial_state.players);
        packet.write(&initial_state.grid_positions);
        packet
    }
}