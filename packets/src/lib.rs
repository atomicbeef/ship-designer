use num_enum::{IntoPrimitive, TryFromPrimitive};
use thiserror::Error;

pub mod bevy_impls;
pub mod primitive_impls;
pub mod tuple_impls;

pub trait PacketSerialize {
    fn serialize(&self, packet: &mut Packet);
}

pub trait PacketDeserialize: Sized {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError>;
}

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    PlacePart,
    DeletePart,
    InitialState,
    PlayerConnected,
    PlayerDisconnected,
    VoxelUpdate,
    SpawnMissile,
    ExplodeMissile,
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
        self.data.extend_from_slice(bytes);
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
        let mut data: Vec<u8> = Vec::with_capacity(packet.data.len() + 1);

        data.push(packet.packet_type().into());
        data.extend_from_slice(&packet.data);

        data.into_boxed_slice()
    }
}