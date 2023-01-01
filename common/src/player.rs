use bevy::prelude::Resource;
use bevy::utils::HashMap;

use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId {
    id: u8
}

impl From<u8> for PlayerId {
    fn from(id: u8) -> Self {
        PlayerId { id }
    }
}

impl PacketSerialize for PlayerId {
    fn serialize(&self, packet: &mut Packet) {
        self.id.serialize(packet);
    }
}

impl PacketDeserialize for PlayerId {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let id = u8::deserialize(packet)?;
        Ok(Self { id })
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    id: PlayerId,
    name: String
}

impl Player {
    pub fn new(id: PlayerId, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl PacketSerialize for Player {
    fn serialize(&self, packet: &mut Packet) {
        self.id().serialize(packet);
        self.name().serialize(packet);
    }
}

impl PacketDeserialize for Player {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let id = PlayerId::deserialize(packet)?;
        let name = String::deserialize(packet)?;
        Ok(Player::new(id, name))
    }
}

#[derive(Resource)]
pub struct Players {
    players: HashMap<PlayerId, Player>
}

impl Players {
    pub fn new() -> Self {
        Self { players: HashMap::new() }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id(), player);
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.remove(&id);
    }

    pub fn player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(&id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &PlayerId> {
        self.players.keys()
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.values()
    }
}