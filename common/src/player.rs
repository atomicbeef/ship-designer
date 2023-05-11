use bevy::prelude::Component;
use packets_derive::{PacketSerialize, PacketDeserialize};

#[derive(Clone, Copy, Debug, Component, PartialEq, Eq, Hash, PacketSerialize, PacketDeserialize)]
pub struct PlayerId {
    id: u8
}

impl From<u8> for PlayerId {
    fn from(id: u8) -> Self {
        PlayerId { id }
    }
}

#[derive(Clone, Debug, Component, PacketSerialize, PacketDeserialize)]
pub struct PlayerName {
    name: String
}

impl From<String> for PlayerName {
    fn from(name: String) -> Self {
        PlayerName { name }
    }
}

impl std::fmt::Display for PlayerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}