use bevy::prelude::Resource;
use uflow::client::Client;

#[derive(Resource)]
pub struct ConnectionState {
    pub client: Client,
    
}

impl ConnectionState {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}