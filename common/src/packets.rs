use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

use crate::{grid::GridPos, player::Player};

pub trait PacketSerialize<T> {
    fn write(&mut self, x: T);
}

pub trait PacketDeserialize<T> {
    fn read(&mut self) -> Result<T, PacketError>;
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    PlaceBlock,
    DeleteBlock,
    InitialState,
    PlayerConnected,
    PlayerDisconnected
}

#[derive(Debug, Clone)]
pub struct Packet {
    data: Vec<u8>,
    index: usize,
    packet_type: PacketType
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Attempted to read too many bytes from packet {0:?}!")]
    BoundsError(Packet),

    #[error("The value of the data at index {} in packet {0:?} is invalid!", .0.index)]
    InvalidPacketError(Packet),

    #[error("The packet type {0} is invalid!")]
    InvalidTypeError(u8),

    #[error("Empty packet!")]
    EmptyPacket
}

impl Packet {
    pub fn new(packet_type: PacketType) -> Self {
        Packet {
            data: Vec::new(),
            index: 0,
            packet_type
        }
    }

    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    pub fn next_bytes(&mut self, num_bytes: usize) -> Result<&[u8], PacketError> {
        if self.index + num_bytes > self.data.len() {
            Err(PacketError::BoundsError(self.clone()))
        } else {
            let bytes = &self.data[self.index..self.index + num_bytes];
            self.index += num_bytes;

            Ok(bytes)
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.data.push(*byte);
        }
    }
}

impl PacketSerialize<u8> for Packet {
    fn write(&mut self, num: u8) {
        self.data.push(num);
    }
}

impl PacketDeserialize<u8> for Packet {
    fn read(&mut self) -> Result<u8, PacketError> {
        if self.index >= self.data.len() {
            Err(PacketError::BoundsError(self.clone()))
        } else {
            let byte = self.data[self.index];
            self.index += 1;
            
            Ok(byte)
        }
    }
}

impl PacketSerialize<i16> for Packet {
    fn write(&mut self, num: i16) {
        self.write_bytes(&num.to_le_bytes());
    }
}

impl PacketDeserialize<i16> for Packet {
    fn read(&mut self) -> Result<i16, PacketError> {
        let i16_bytes = self.next_bytes(2)?;
        Ok(i16::from_le_bytes(i16_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize<u16> for Packet {
    fn write(&mut self, num: u16) {
        self.write_bytes(&num.to_le_bytes());
    }
}

impl PacketDeserialize<u16> for Packet {
    fn read(&mut self) -> Result<u16, PacketError> {
        let u16_bytes = self.next_bytes(2)?;
        Ok(u16::from_le_bytes(u16_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize<u32> for Packet {
    fn write(&mut self, num: u32) {
        self.write_bytes(&num.to_le_bytes());
    }
}

impl PacketDeserialize<u32> for Packet {
    fn read(&mut self) -> Result<u32, PacketError> {
        let u32_bytes = self.next_bytes(4)?;
        Ok(u32::from_le_bytes(u32_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize<u64> for Packet {
    fn write(&mut self, num: u64) {
        self.write_bytes(&num.to_le_bytes());
    }
}

impl PacketDeserialize<u64> for Packet {
    fn read(&mut self) -> Result<u64, PacketError> {
        let u64_bytes = self.next_bytes(8)?;
        Ok(u64::from_le_bytes(u64_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize<&String> for Packet {
    // A maximum of 2^16 characters will be written
    // Any remaining characters in the string will silently be ignored
    fn write(&mut self, string: &String) {
        self.write(string.len() as u16);
        self.write_bytes(string.as_bytes());
    }
}

impl PacketDeserialize<String> for Packet {
    fn read(&mut self) -> Result<String, PacketError> {
        let string_len: u16 = self.read()?;
        let string_bytes = self.next_bytes(string_len.into())?;
        
        Ok(String::from_utf8_lossy(string_bytes).into_owned())
    }
}

impl PacketSerialize<&Player> for Packet {
    fn write(&mut self, player: &Player) {
        self.write(player.id());
        self.write(player.name());
    }
}

impl PacketDeserialize<Player> for Packet {
    fn read(&mut self) -> Result<Player, PacketError> {
        let id: u8 = self.read()?;
        let name: String = self.read()?;
        Ok(Player::new(id, name))
    }
}

impl PacketSerialize<&Vec<GridPos>> for Packet {
    fn write(&mut self, positions: &Vec<GridPos>) {
        self.write(positions.len() as u32);
        for position in positions.iter() {
            self.write(*position);
        }
    }
}

impl PacketDeserialize<Vec<GridPos>> for Packet {
    fn read(&mut self) -> Result<Vec<GridPos>, PacketError> {
        let length: u32 = self.read()?;
        let mut positions: Vec<GridPos> = Vec::with_capacity(length as usize);
        for _ in 0..length {
            positions.push(self.read()?);
        }

        Ok(positions)
    }
}

impl PacketSerialize<&Vec<Player>> for Packet {
    fn write(&mut self, players: &Vec<Player>) {
        self.write(players.len() as u32);
        for player in players.iter() {
            self.write(player);
        }
    }
}

impl PacketDeserialize<Vec<Player>> for Packet {
    fn read(&mut self) -> Result<Vec<Player>, PacketError> {
        let length: u32 = self.read()?;
        let mut players: Vec<Player> = Vec::with_capacity(length as usize);
        for _ in 0..length {
            players.push(self.read()?);
        }

        Ok(players)
    }
}

impl TryFrom<Box<[u8]>> for Packet {
    type Error = PacketError;

    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        if data.len() == 0 {
            return Err(PacketError::EmptyPacket)
        }

        let packet_type_u8 = data[0];

        match PacketType::try_from(packet_type_u8) {
            // The index is 1 because the first byte of the packet is the type, which has already been read
            Ok(packet_type) => Ok(Packet { data: data.to_vec(), index: 1, packet_type }),
            Err(_) => Err(PacketError::InvalidTypeError(packet_type_u8))
        }
    }
}

impl From<&Packet> for Box<[u8]> {
    fn from(packet: &Packet) -> Self {
        let mut data: Vec<u8>= Vec::with_capacity(packet.data.len() + 1);
        
        data.push(packet.packet_type().into());
        for byte in packet.data.iter() {
            data.push(*byte);
        }

        data.into_boxed_slice()
    }
}