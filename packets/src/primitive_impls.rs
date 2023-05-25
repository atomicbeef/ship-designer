use super::{Packet, PacketSerialize, PacketDeserialize, PacketError};

impl<T: PacketSerialize> PacketSerialize for &T {
    fn serialize(&self, packet: &mut Packet) {
        (**self).serialize(packet);
    }
}

impl PacketSerialize for u8 {
    fn serialize(&self, packet: &mut Packet) {
        packet.data.push(*self);
    }
}

impl PacketDeserialize for u8 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        if packet.index >= packet.data.len() {
            Err(PacketError::BoundsError(packet.clone()))
        } else {
            let byte = packet.data[packet.index];
            packet.index += 1;

            Ok(byte)
        }
    }
}

impl PacketSerialize for bool {
    fn serialize(&self, packet: &mut Packet) {
        (*self as u8).serialize(packet);
    }
}

impl PacketDeserialize for bool {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let val_bytes = u8::deserialize(packet)?;
        match val_bytes {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PacketError::InvalidPacketError(packet.clone()))
        }
    }
}

impl PacketSerialize for i16 {
    fn serialize(&self, packet: &mut Packet) {
        packet.write_bytes(&self.to_le_bytes());
    }
}

impl PacketDeserialize for i16 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let i16_bytes = packet.next_bytes(2)?;
        Ok(i16::from_le_bytes(i16_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize for u16 {
    fn serialize(&self, packet: &mut Packet) {
        packet.write_bytes(&self.to_le_bytes());
    }
}

impl PacketDeserialize for u16 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let u16_bytes = packet.next_bytes(2)?;
        Ok(u16::from_le_bytes(u16_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize for u32 {
    fn serialize(&self, packet: &mut Packet) {
        packet.write_bytes(&self.to_le_bytes());
    }
}

impl PacketDeserialize for u32 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let u32_bytes = packet.next_bytes(4)?;
        Ok(u32::from_le_bytes(u32_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize for u64 {
    fn serialize(&self, packet: &mut Packet) {
        packet.write_bytes(&self.to_le_bytes());
    }
}

impl PacketDeserialize for u64 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let u64_bytes = packet.next_bytes(8)?;
        Ok(u64::from_le_bytes(u64_bytes.try_into().unwrap()))
    }
}

impl PacketSerialize for String {
    // A maximum of 2^16 characters will be written
    // Any remaining characters in the string will silently be ignored
    fn serialize(&self, packet: &mut Packet) {
        (self.len() as u16).serialize(packet);
        packet.write_bytes(self.as_bytes());
    }
}

impl PacketDeserialize for String {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let string_len = u16::deserialize(packet)?;
        let string_bytes = packet.next_bytes(string_len.into())?;

        Ok(String::from_utf8_lossy(string_bytes).into_owned())
    }
}

impl PacketSerialize for f32 {
    fn serialize(&self, packet: &mut Packet) {
        packet.write_bytes(&self.to_le_bytes());
    }
}

impl PacketDeserialize for f32 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let float = packet.next_bytes(4)?;
        Ok(f32::from_le_bytes(float.try_into().unwrap()))
    }
}

impl<T: PacketSerialize> PacketSerialize for [T] {
    fn serialize(&self, packet: &mut Packet) {
        (self.len() as u64).serialize(packet);
        for elem in self {
            elem.serialize(packet);
        }
    }
}

impl<T: PacketSerialize> PacketSerialize for &[T] {
    fn serialize(&self, packet: &mut Packet) {
        (self.len() as u64).serialize(packet);
        for elem in self.into_iter() {
            elem.serialize(packet);
        }
    }
}

impl<T: PacketSerialize> PacketSerialize for Vec<T> {
    fn serialize(&self, packet: &mut Packet) {
        (self.len() as u64).serialize(packet);
        for elem in self {
            elem.serialize(packet);
        }
    }
}

impl<T: PacketDeserialize> PacketDeserialize for Vec<T> {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let len = u64::deserialize(packet)? as usize;
        let mut vector = Vec::with_capacity(len);

        for _ in 0..len {
            let elem = T::deserialize(packet)?;
            vector.push(elem);
        }

        Ok(vector)
    }
}

impl<T: PacketSerialize> PacketSerialize for Option<T> {
    fn serialize(&self, packet: &mut Packet) {
        match self {
            Self::Some(inner) => {
                true.serialize(packet);
                inner.serialize(packet);
            },
            Self::None => {
                false.serialize(packet);
            }
        }
    }
}

impl<T: PacketDeserialize> PacketDeserialize for Option<T> {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let has_some = bool::deserialize(packet)?;

        if has_some {
            let inner = T::deserialize(packet)?;
            Ok(Some(inner))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Packet, PacketType::PlayerConnected, PacketSerialize, PacketDeserialize};

    #[test]
    fn u8_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: u8 = 12;
        x.serialize(&mut packet);

        let y = u8::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn bool_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x = true;
        x.serialize(&mut packet);

        let y = bool::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn i16_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: i16 = 12;
        x.serialize(&mut packet);

        let y = i16::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }
    
    #[test]
    fn u16_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: u16 = 12;
        x.serialize(&mut packet);

        let y = u16::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn u32_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: u32 = 12;
        x.serialize(&mut packet);

        let y = u32::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn u64_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: u64 = 12;
        x.serialize(&mut packet);

        let y = u64::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn string_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x: String = "Test".to_string();
        x.serialize(&mut packet);

        let y = String::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn f32_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);
        
        let x: f32 = 12.0;
        x.serialize(&mut packet);

        let y = f32::deserialize(&mut packet).unwrap();

        assert_eq!(x.to_le_bytes(), y.to_le_bytes());
    }

    #[test]
    fn slice_serialize_vec_deserialize() {
        let mut packet = Packet::new(PlayerConnected);
        
        let x = [1.0, 2.0, 3.0];
        x.serialize(&mut packet);

        let y: Vec<f32> = Vec::deserialize(&mut packet).unwrap();

        for i in 0..x.len() {
            assert_eq!(x[i].to_le_bytes(), y[i].to_le_bytes());
        }
    }

    #[test]
    fn vec_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);
        
        let x = vec![1.0, 2.0, 3.0];
        x.serialize(&mut packet);

        let y: Vec<f32> = Vec::deserialize(&mut packet).unwrap();

        for i in 0..x.len() {
            assert_eq!(x[i].to_le_bytes(), y[i].to_le_bytes());
        }
    }

    #[test]
    fn option_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);
        
        let x = Some(1.0);
        x.serialize(&mut packet);

        let y: Option<f32> = Option::deserialize(&mut packet).unwrap();

        assert_eq!(x.unwrap().to_le_bytes(), y.unwrap().to_le_bytes());
    }
}