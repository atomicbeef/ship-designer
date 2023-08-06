use bevy::prelude::*;

use crate::network_id::NetworkId;
use packets_derive::{IntoPacket, TryFromPacket};
use crate::player::{PlayerName, PlayerId};
use crate::part::PartNetworkRepr;
use crate::compact_transform::CompactTransform;

#[derive(IntoPacket, TryFromPacket, Event)]
#[PacketType(PlayerConnected)]
pub struct PlayerConnected {
    pub id: PlayerId,
    pub name: PlayerName,
    pub transform: Transform,
}

#[derive(IntoPacket, TryFromPacket, Event)]
#[PacketType(PlayerDisconnected)]
pub struct PlayerDisconnected(pub PlayerId);

#[derive(IntoPacket, TryFromPacket, Event)]
#[PacketType(InitialState)]
pub struct InitialState {
    pub player_id: PlayerId,
    pub players: Vec<(PlayerId, PlayerName, Transform)>,
    pub construct_network_id: NetworkId,
    pub parts: Vec<(PartNetworkRepr, CompactTransform, NetworkId)>,
    pub construct_transform: CompactTransform
}