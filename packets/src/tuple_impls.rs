use super::{Packet, PacketSerialize, PacketDeserialize, PacketError};

impl<A> PacketSerialize for (A,) 
where
    A: PacketSerialize
{
    fn serialize(&self, packet: &mut Packet) {
        self.0.serialize(packet);
    }
}

impl <A> PacketDeserialize for (A,) 
where 
    A: PacketDeserialize
{
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let a = A::deserialize(packet)?;
        Ok((a,))
    }
}

impl<A, B> PacketSerialize for (A, B) 
where 
    A: PacketSerialize,
    B: PacketSerialize
{
    fn serialize(&self, packet: &mut Packet) {
        self.0.serialize(packet);
        self.1.serialize(packet);
    }
}

impl <A, B> PacketDeserialize for (A, B) 
where 
    A: PacketDeserialize,
    B: PacketDeserialize
{
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let a = A::deserialize(packet)?;
        let b = B::deserialize(packet)?;
        Ok((a, b))
    }
}

impl<A, B, C> PacketSerialize for (A, B, C) 
where 
    A: PacketSerialize,
    B: PacketSerialize,
    C: PacketSerialize
{
    fn serialize(&self, packet: &mut Packet) {
        self.0.serialize(packet);
        self.1.serialize(packet);
        self.2.serialize(packet);
    }
}

impl <A, B, C> PacketDeserialize for (A, B, C) 
where 
    A: PacketDeserialize,
    B: PacketDeserialize,
    C: PacketDeserialize
{
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let a = A::deserialize(packet)?;
        let b = B::deserialize(packet)?;
        let c = C::deserialize(packet)?;
        Ok((a, b, c))
    }
}