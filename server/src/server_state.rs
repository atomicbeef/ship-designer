use uflow::{Server, Peer};

pub struct ServerState {
    pub server: Server,
    pub peer_list: Vec<Peer>,
}