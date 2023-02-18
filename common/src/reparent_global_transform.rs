use bevy::prelude::{Transform, GlobalTransform};

pub trait ReparentGlobalTransform {
    fn reparented_to(&self, parent: &GlobalTransform) -> Transform;
}

impl ReparentGlobalTransform for GlobalTransform {
    fn reparented_to(&self, parent: &GlobalTransform) -> Transform {
        let relative_affine = parent.affine().inverse() * self.affine();
        let (scale, rotation, translation) = relative_affine.to_scale_rotation_translation();
        Transform {
            translation,
            rotation,
            scale,
        }
    }
}