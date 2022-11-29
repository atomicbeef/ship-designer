use bevy::prelude::{Handle, Resource};
use bevy::render::mesh::Mesh;
use bevy::utils::HashMap;

use common::shape::ShapeHandle;

#[derive(Resource)]
pub struct MeshHandles {
    mesh_handles: HashMap<ShapeHandle, Handle<Mesh>>
}

impl MeshHandles {
    pub fn new() -> Self {
        Self { mesh_handles: HashMap::new() }
    }

    pub fn get(&self, shape_handle: &ShapeHandle) -> Option<&Handle<Mesh>> {
        self.mesh_handles.get(shape_handle)
    }

    pub fn add(&mut self, shape_handle: ShapeHandle, mesh_handle: Handle<Mesh>) {
        self.mesh_handles.insert(shape_handle, mesh_handle);
    }
}