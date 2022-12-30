use crate::packets::{Packet, PacketSerialize, PacketDeserialize, PacketError};

#[derive(Clone, Debug)]
pub struct Player {
    id: u8,
    name: String
}

impl Player {
    pub fn new(id: u8, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> u8 {
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
        let id = u8::deserialize(packet)?;
        let name = String::deserialize(packet)?;
        Ok(Player::new(id, name))
    }
}