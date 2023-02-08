use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    reflect::TypeUuid
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "2e56bfa6-2ef6-45c5-8a93-5bcda48be051"]
pub struct BuildingMaterial {
    #[uniform(0)]
    pub color: Color
}

impl Material for BuildingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/building_material.wgsl".into()
    }
}