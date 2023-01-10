use bevy::prelude::{Component, Entity};
use bevy::ecs::query::QueryIter;

use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
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

impl PacketSerialize for NetworkId {
    fn serialize(&self, packet: &mut Packet) {
        self.id.serialize(packet);
    }
}

impl PacketDeserialize for NetworkId {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let id = u32::deserialize(packet)?;
        Ok(Self::from(id))
    }
}

pub fn entity_from_network_id(
    network_id_iterator: QueryIter<(Entity, &NetworkId), ()>,
    network_id: NetworkId
) -> Option<Entity> {
    for (entity, id) in network_id_iterator {
        if *id == network_id {
            return Some(entity);
        }
    }

    None
}