use bevy::utils::hashbrown::{hash_map, HashMap};
use uflow::{Server, Peer};

use common::player::Player;

pub struct ServerState {
    pub server: Server,
    last_player_id: u8,
    players: HashMap<u8, (Player, Peer)>,
}

impl ServerState {
    pub fn new(server: Server) -> Self {
        Self { server, last_player_id: 0, players: HashMap::new() }
    }

    pub fn add_player(&mut self, peer: Peer, name: String) {
        loop {
            self.last_player_id = self.last_player_id.wrapping_add(1);
            if let None = self.players.get(&self.last_player_id) {
                break;
            }
        }

        let player = Player::new(self.last_player_id, name);

        self.players.insert(self.last_player_id, (player, peer));
    }

    pub fn remove_player(&mut self, id: u8) {
        self.players.remove(&id);
    }

    pub fn players(&self) -> impl Iterator<Item=&Player> {
        self.players.values().map(|x| &x.0)
    }

    pub fn peer_mut(&mut self, id: u8) -> Option<&mut Peer> {
        match self.players.get_mut(&id) {
            Some(pair) => Some(&mut pair.1),
            None => None
        }
    }

    pub fn peers_mut(&mut self) -> impl Iterator<Item=&mut Peer> {
        self.players.values_mut().map(|x| &mut x.1)
    }

    pub fn players_peers_mut(&mut self) -> hash_map::ValuesMut<'_, u8, (Player, Peer)> {
        self.players.values_mut()
    }
}