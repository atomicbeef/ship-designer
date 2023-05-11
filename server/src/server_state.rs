use std::net::SocketAddr;

use bevy::utils::HashMap;
use uflow::SendMode;
use uflow::server::Server;

use common::player::PlayerId;

pub struct ServerState {
    pub server: Server,
    current_player_id: u8,
    client_addresses: HashMap<PlayerId, SocketAddr>,
    player_ids: HashMap<SocketAddr, PlayerId>
}

impl ServerState {
    pub fn new(server: Server) -> Self {
        Self { server, current_player_id: 0, client_addresses: HashMap::new(), player_ids: HashMap::new() }
    }

    pub fn send_to_player(
        &mut self,
        player_id: PlayerId,
        data: Box<[u8]>,
        channel_id: usize,
        send_mode: SendMode
    ) {
        if let Some(client_address) = self.client_addresses.get(&player_id) {
            if let Some(remote_client) = self.server.client(client_address) {
                remote_client.borrow_mut().send(data, channel_id, send_mode);
            }
        }
    }

    pub fn new_player_id(&mut self) -> PlayerId {
        let id = self.current_player_id;
        self.set_next_id();

        PlayerId::from(id)
    }

    fn set_next_id(&mut self) {
        loop {
            self.current_player_id = self.current_player_id.wrapping_add(1);
            if let None = self.client_addresses.get(&self.current_player_id.into()) {
                break;
            }
        }
    }

    pub fn add_client_address(&mut self, player_id: PlayerId, client_address: SocketAddr) {
        self.client_addresses.insert(player_id, client_address);
        self.player_ids.insert(client_address, player_id);
    }

    pub fn remove_client_address(&mut self, player_id: PlayerId) {
        if let Some(address) = self.client_addresses.remove(&player_id) {
            self.player_ids.remove(&address);
        }
    }

    pub fn player_id(&self, client_address: SocketAddr) -> Option<&PlayerId> {
        self.player_ids.get(&client_address)
    }
}