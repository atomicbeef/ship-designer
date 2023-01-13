use bevy::log::debug;
use bevy::prelude::{Handle, Resource, EventReader, ResMut};
use bevy::render::mesh::Mesh;
use bevy::utils::HashMap;

use common::shape::{ShapeId, FreedShapes};

#[derive(Resource)]
pub struct MeshHandles {
    mesh_handles: HashMap<ShapeId, Handle<Mesh>>
}

impl MeshHandles {
    pub fn new() -> Self {
        Self { mesh_handles: HashMap::new() }
    }

    pub fn get(&self, shape_id: &ShapeId) -> Option<&Handle<Mesh>> {
        self.mesh_handles.get(shape_id)
    }

    pub fn add(&mut self, shape_id: ShapeId, mesh_handle: Handle<Mesh>) {
        self.mesh_handles.insert(shape_id, mesh_handle);
    }

    pub fn update(&mut self, shape_id: ShapeId, mesh_handle: Handle<Mesh>) {
        self.mesh_handles.entry(shape_id).and_modify(|e| *e = mesh_handle);
    }
}

pub fn free_mesh_handles(mut freed_shapes_reader: EventReader<FreedShapes>, mut mesh_handles: ResMut<MeshHandles>) {
    for freed_shapes in freed_shapes_reader.iter() {
        for id in freed_shapes.0.iter() {
            mesh_handles.mesh_handles.remove(id);
            debug!("Removed mesh handle for shape ID {:?}", id);
        }
    }
}