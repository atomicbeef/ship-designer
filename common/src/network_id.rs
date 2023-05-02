use bevy::prelude::*;

use packets_derive::{PacketSerialize, PacketDeserialize};

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq, Hash, PacketSerialize, PacketDeserialize)]
pub struct NetworkId {
    id: u32
}

impl NetworkId {
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl From<u32> for NetworkId {
    fn from(id: u32) -> Self {
        Self { id }
    }
}