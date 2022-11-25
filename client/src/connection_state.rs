use bevy::{utils::hashbrown::HashMap, prelude::Resource};
use uflow::{Client, Peer};

use common::player::Player;

#[derive(Resource)]
pub struct ConnectionState {
    pub client: Client,
    pub server: Peer,
    players: HashMap<u8, Player>
}

impl ConnectionState {
    pub fn new(client: Client, server: Peer) -> Self {
        Self { client, server, players: HashMap::new() }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id(), player);
    }

    pub fn remove_player(&mut self, id: u8) {
        self.players.remove(&id);
    }

    pub fn player(&self, id: u8) -> Option<&Player> {
        self.players.get(&id)
    }
}