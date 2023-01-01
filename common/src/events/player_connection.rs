use crate::network_id::NetworkId;
use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError, PacketType};
use crate::player::{Player, PlayerId};
use crate::shape::ShapeNetworkRepr;
use crate::shape_transform::ShapeTransform;

pub struct PlayerConnected {
    pub id: PlayerId,
    pub name: String
}

impl TryFrom<Packet> for PlayerConnected {
    type Error = PacketError;
    
    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = PlayerId::deserialize(&mut packet)?;
        let name = String::deserialize(&mut packet)?;
        Ok(Self { id, name })
    }
}

impl From<&PlayerConnected> for Packet {
    fn from(player_connected: &PlayerConnected) -> Self {
        let mut packet = Packet::new(PacketType::PlayerConnected);
        player_connected.id.serialize(&mut packet);
        (&player_connected.name).serialize(&mut packet);
        packet
    }
}

pub struct PlayerDisconnected(pub PlayerId);

impl TryFrom<Packet> for PlayerDisconnected {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let id = PlayerId::deserialize(&mut packet)?;
        Ok(Self(id))
    }
}

impl From<&PlayerDisconnected> for Packet {
    fn from(player_disconnected: &PlayerDisconnected) -> Self {
        let mut packet = Packet::new(PacketType::PlayerDisconnected);
        player_disconnected.0.serialize(&mut packet);
        packet
    }
}

pub struct InitialState {
    pub players: Vec<Player>,
    pub shapes: Vec<(ShapeNetworkRepr, ShapeTransform, NetworkId)>,
}

impl TryFrom<Packet> for InitialState {
    type Error = PacketError;

    fn try_from(mut packet: Packet) -> Result<Self, Self::Error> {
        let players: Vec<Player> = Vec::deserialize(&mut packet)?;
        let shapes: Vec<(ShapeNetworkRepr, ShapeTransform, NetworkId)> = Vec::deserialize(&mut packet)?;
        Ok(Self { players, shapes })
    }
}

impl From<&InitialState> for Packet {
    fn from(initial_state: &InitialState) -> Self {
        let mut packet = Packet::new(PacketType::InitialState);
        (&initial_state.players).serialize(&mut packet);
        (&initial_state.shapes).serialize(&mut packet);
        packet
    }
}