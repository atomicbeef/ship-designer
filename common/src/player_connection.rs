use crate::network_id::NetworkId;
use packets_derive::{IntoPacket, TryFromPacket};
use crate::player::{Player, PlayerId};
use crate::part::PartNetworkRepr;
use crate::compact_transform::CompactTransform;

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(PlayerConnected)]
pub struct PlayerConnected {
    pub id: PlayerId,
    pub name: String
}

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(PlayerDisconnected)]
pub struct PlayerDisconnected(pub PlayerId);

#[derive(IntoPacket, TryFromPacket)]
#[PacketType(InitialState)]
pub struct InitialState {
    pub players: Vec<Player>,
    pub construct_network_id: NetworkId,
    pub parts: Vec<(PartNetworkRepr, CompactTransform, NetworkId)>,
    pub construct_transform: CompactTransform
}