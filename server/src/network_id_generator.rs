use bevy::prelude::Resource;
use common::network_id::NetworkId;

#[derive(Resource)]
pub struct NetworkIdGenerator {
    current_id: u32
}

impl NetworkIdGenerator {
    pub fn new() -> Self {
        Self { current_id: 0 }
    }

    pub fn generate(&mut self) -> NetworkId {
        let network_id = NetworkId::from(self.current_id);
        self.current_id += 1;
        
        network_id
    }
}