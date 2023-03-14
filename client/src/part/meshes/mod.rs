use bevy::log::debug;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy::utils::HashMap;

use common::part::{Part, PartId, FreedParts};

use self::mesh_generation::generate_part_mesh;

pub mod mesh_generation;

#[derive(Resource)]
pub struct PartMeshHandles {
    mesh_handles: HashMap<PartId, Handle<Mesh>>
}

impl PartMeshHandles {
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

pub fn free_part_mesh_handles(mut freed_parts_reader: EventReader<FreedParts>, mut mesh_handles: ResMut<PartMeshHandles>) {
    for freed_parts in freed_parts_reader.iter() {
        for id in freed_parts.0.iter() {
            mesh_handles.mesh_handles.remove(id);
            debug!("Removed mesh handle for part ID {:?}", id);
        }
    }
}

pub fn get_mesh_or_generate(
    part_id: PartId,
    part: &Part,
    mesh_handles: &mut PartMeshHandles,
    meshes: &mut Assets<Mesh>
) -> Handle<Mesh> {
    match mesh_handles.get(&part_id) {
        Some(mesh_handle) => mesh_handle.clone(),
        None => {
            let mesh = generate_part_mesh(part);

            let mesh_handle = meshes.add(mesh);
            mesh_handles.add(part_id, mesh_handle.clone());

            mesh_handle
        }
    }
}