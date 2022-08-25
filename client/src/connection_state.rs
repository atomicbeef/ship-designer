use uflow::{Client, Peer};

pub struct OtherPeer {
    pub name: String
}

pub struct ConnectionState {
    pub client: Client,
    pub server: Peer,
    pub other_peers: Vec<OtherPeer>
}