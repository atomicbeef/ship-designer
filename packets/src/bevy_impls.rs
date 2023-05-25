use bevy::prelude::{Transform, Vec3, Quat};

use super::{Packet, PacketSerialize, PacketDeserialize, PacketError};

impl PacketSerialize for Vec3 {
    fn serialize(&self, packet: &mut Packet) {
        self.to_array().serialize(packet);
    }
}

impl PacketDeserialize for Vec3 {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let vector: Vec<f32> = Vec::deserialize(packet)?;
        Ok(Vec3::from_slice(&vector))
    }
}

impl PacketSerialize for Quat {
    fn serialize(&self, packet: &mut Packet) {
        self.to_array().serialize(packet);
    }
}

impl PacketDeserialize for Quat {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let quat: Vec<f32> = Vec::deserialize(packet)?;
        Ok(Quat::from_slice(&quat))
    }
}

impl PacketSerialize for Transform {
    fn serialize(&self, packet: &mut Packet) {
        self.translation.serialize(packet);
        self.rotation.serialize(packet);
        self.scale.serialize(packet);
    }
}

impl PacketDeserialize for Transform {
    fn deserialize(packet: &mut Packet) -> Result<Self, PacketError> {
        let translation = Vec3::deserialize(packet)?;
        let rotation = Quat::deserialize(packet)?;
        let scale = Vec3::deserialize(packet)?;
        Ok(Transform { translation, rotation, scale })
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Vec3, Quat, Transform};

    use crate::{Packet, PacketSerialize, PacketDeserialize};
    use crate::PacketType::PlayerConnected;

    #[test]
    fn vec3_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x = Vec3::new(1.0, -5.0, 0.0);
        x.serialize(&mut packet);

        let y = Vec3::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn quat_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x = Quat::from_rotation_x(1.0);
        x.serialize(&mut packet);

        let y = Quat::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }

    #[test]
    fn transform_serialize_deserialize() {
        let mut packet = Packet::new(PlayerConnected);

        let x = Transform {
            translation: Vec3::new(1.0, -5.0, 0.0),
            rotation: Quat::from_rotation_x(1.0),
            scale: Vec3::new(5.0, 1.25, 3.0)
        };
        x.serialize(&mut packet);

        let y = Transform::deserialize(&mut packet).unwrap();
        
        assert_eq!(x, y);
    }
}