use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq, Hash)]
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

#[derive(Resource)]
pub struct NetworkIdIndex {
    network_id_index: HashMap<NetworkId, Entity>,
    // Needed to be able to remove dropped entities from the index
    entity_index: HashMap<Entity, NetworkId>
}

impl NetworkIdIndex {
    pub fn new() -> Self {
        Self { network_id_index: HashMap::new(), entity_index: HashMap::new() }
    }

    pub fn entity(&self, network_id: &NetworkId) -> Option<Entity> {
        self.network_id_index.get(network_id).copied()
    }

    pub fn insert(&mut self, network_id: NetworkId, entity: Entity) {
        self.network_id_index.insert(network_id, entity);
        self.entity_index.insert(entity, network_id);
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(network_id) = self.entity_index.remove(&entity) {
            self.network_id_index.remove(&network_id);
        }
    }
}

pub fn update_index(
    mut network_id_index: ResMut<NetworkIdIndex>,
    added_query: Query<(Entity, &NetworkId), Added<NetworkId>>,
    removed_query: RemovedComponents<NetworkId>
) {
    for (entity, new_network_id) in added_query.iter() {
        network_id_index.insert(*new_network_id, entity);
    }

    for entity in removed_query.iter() {
        network_id_index.remove(entity);
    }
}