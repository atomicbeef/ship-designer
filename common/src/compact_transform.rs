use bevy::prelude::{Transform, Vec3, Quat, GlobalTransform};
use packets_derive::{PacketSerialize, PacketDeserialize};

#[derive(Clone, Copy, Debug, PacketSerialize, PacketDeserialize)]
pub struct CompactTransform {
    pub translation: Vec3,
    pub rotation: Quat
}

impl CompactTransform {
    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        CompactTransform { translation, rotation }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        CompactTransform { translation: Vec3::new(x, y, z), rotation: Quat::IDENTITY }
    }
}

impl From<CompactTransform> for Transform {
    fn from(value: CompactTransform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation,
            scale: Vec3::splat(1.0)
        }
    }
}

impl From<Transform> for CompactTransform {
    fn from(value: Transform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation
        }
    }
}

impl From<GlobalTransform> for CompactTransform {
    fn from(value: GlobalTransform) -> Self {
        let (_, rotation, translation) = value.to_scale_rotation_translation();
        
        Self {
            translation,
            rotation,
        }
    }
}