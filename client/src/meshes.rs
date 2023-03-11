use bevy::log::debug;
use bevy::prelude::{Handle, Resource, EventReader, ResMut};
use bevy::render::mesh::Mesh;
use bevy::utils::HashMap;

use common::part::{PartId, FreedParts};

#[derive(Resource)]
pub struct MeshHandles {
    mesh_handles: HashMap<PartId, Handle<Mesh>>
}

impl MeshHandles {
    pub fn new() -> Self {
        Self { mesh_handles: HashMap::new() }
    }

    pub fn get(&self, part_id: &PartId) -> Option<&Handle<Mesh>> {
        self.mesh_handles.get(part_id)
    }

    pub fn add(&mut self, part_id: PartId, mesh_handle: Handle<Mesh>) {
        self.mesh_handles.insert(part_id, mesh_handle);
    }

    pub fn update(&mut self, part_id: PartId, mesh_handle: Handle<Mesh>) {
        self.mesh_handles.entry(part_id).and_modify(|e| *e = mesh_handle);
    }
}

pub fn free_mesh_handles(mut freed_parts_reader: EventReader<FreedParts>, mut mesh_handles: ResMut<MeshHandles>) {
    for freed_parts in freed_parts_reader.iter() {
        for id in freed_parts.0.iter() {
            mesh_handles.mesh_handles.remove(id);
            debug!("Removed mesh handle for part ID {:?}", id);
        }
    }
}