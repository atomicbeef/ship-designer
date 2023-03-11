use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use common::part::{Part, PartHandle, Parts, VOXEL_SIZE, PartId};
use common::materials::Material;

use crate::meshes::MeshHandles;

fn add_box_mesh_data(
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
    vertices: &mut Vec<[f32; 3]>,
    triangles: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    index_offset: &mut u32
) {
    let verts = &[
        // Front
        ([min_x, min_y, max_z], [0., 0., 1.0], [0., 0.]),
        ([max_x, min_y, max_z], [0., 0., 1.0], [1.0, 0.]),
        ([max_x, max_y, max_z], [0., 0., 1.0], [1.0, 1.0]),
        ([min_x, max_y, max_z], [0., 0., 1.0], [0., 1.0]),
        // Back
        ([min_x, max_y, min_z], [0., 0., -1.0], [1.0, 0.]),
        ([max_x, max_y, min_z], [0., 0., -1.0], [0., 0.]),
        ([max_x, min_y, min_z], [0., 0., -1.0], [0., 1.0]),
        ([min_x, min_y, min_z], [0., 0., -1.0], [1.0, 1.0]),
        // Right
        ([max_x, min_y, min_z], [1.0, 0., 0.], [0., 0.]),
        ([max_x, max_y, min_z], [1.0, 0., 0.], [1.0, 0.]),
        ([max_x, max_y, max_z], [1.0, 0., 0.], [1.0, 1.0]),
        ([max_x, min_y, max_z], [1.0, 0., 0.], [0., 1.0]),
        // Left
        ([min_x, min_y, max_z], [-1.0, 0., 0.], [1.0, 0.]),
        ([min_x, max_y, max_z], [-1.0, 0., 0.], [0., 0.]),
        ([min_x, max_y, min_z], [-1.0, 0., 0.], [0., 1.0]),
        ([min_x, min_y, min_z], [-1.0, 0., 0.], [1.0, 1.0]),
        // Top
        ([max_x, max_y, min_z], [0., 1.0, 0.], [1.0, 0.]),
        ([min_x, max_y, min_z], [0., 1.0, 0.], [0., 0.]),
        ([min_x, max_y, max_z], [0., 1.0, 0.], [0., 1.0]),
        ([max_x, max_y, max_z], [0., 1.0, 0.], [1.0, 1.0]),
        // Bottom
        ([max_x, min_y, max_z], [0., -1.0, 0.], [0., 0.]),
        ([min_x, min_y, max_z], [0., -1.0, 0.], [1.0, 0.]),
        ([min_x, min_y, min_z], [0., -1.0, 0.], [1.0, 1.0]),
        ([max_x, min_y, min_z], [0., -1.0, 0.], [0., 1.0]),
    ];

    vertices.extend(verts.iter().map(|(p, _, _)| *p));
    normals.extend(verts.iter().map(|(_, n, _)| *n));
    uvs.extend(verts.iter().map(|(_, _, uv)| *uv));

    triangles.extend([
        // Front
        *index_offset, *index_offset + 1, *index_offset + 2, *index_offset + 2, *index_offset + 3, *index_offset,
        // Back
        *index_offset + 4, *index_offset + 5, *index_offset + 6, *index_offset + 6, *index_offset + 7, *index_offset + 4,
        // Right
        *index_offset + 8, *index_offset + 9, *index_offset + 10, *index_offset + 10, *index_offset + 11, *index_offset + 8,
        // Left
        *index_offset + 12, *index_offset + 13, *index_offset + 14, *index_offset + 14, *index_offset + 15, *index_offset + 12,
        // Top
        *index_offset + 16, *index_offset + 17, *index_offset + 18, *index_offset + 18, *index_offset + 19, *index_offset + 16,
        // Bottom
        *index_offset + 20, *index_offset + 21, *index_offset + 22, *index_offset + 22, *index_offset + 23, *index_offset + 20
    ]);

    *index_offset += 24;
}

pub fn generate_part_mesh(
    part: &Part
) -> Mesh {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut index_offset: u32 = 0;
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for s_x in 0..part.width() {
        for s_y in 0..part.height() {
            for s_z in 0..part.depth() {
                if matches!(part.get(s_x, s_y, s_z), Material::Empty) {
                    continue;
                }

                let x = s_x as f32 * VOXEL_SIZE;
                let y = s_y as f32 * VOXEL_SIZE;
                let z = s_z as f32 * VOXEL_SIZE;

                add_box_mesh_data(
                    x,
                    x + VOXEL_SIZE,
                    y,
                    y + VOXEL_SIZE,
                    z,
                    z + VOXEL_SIZE,
                    &mut vertices,
                    &mut triangles,
                    &mut normals,
                    &mut uvs,
                    &mut index_offset
                );
            }
        }
    }

    // Center mesh to align with colliders
    for vertex in vertices.iter_mut() {
        vertex[0] -= part.width() as f32 * VOXEL_SIZE / 2.0;
        vertex[1] -= part.height() as f32 * VOXEL_SIZE / 2.0;
        vertex[2] -= part.depth() as f32 * VOXEL_SIZE / 2.0;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(triangles)));

    mesh
}

pub struct RegeneratePartMesh(pub Entity);

pub fn regenerate_part_mesh(
    mut regenerate_part_mesh_reader: EventReader<RegeneratePartMesh>,
    part_handle_query: Query<&PartHandle>,
    parts: Res<Parts>,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands
) {
    for request in regenerate_part_mesh_reader.iter() {
        if let Ok(part_handle) = part_handle_query.get(request.0) {
            if let Some(part) = parts.get(part_handle) {
                let mesh = generate_part_mesh(part);
                let mesh_handle = meshes.add(mesh);

                mesh_handles.update(part_handle.id(), mesh_handle.clone());
                
                commands.entity(request.0).remove::<Handle<Mesh>>();
                commands.entity(request.0).insert(mesh_handle);
            }
        }
    }
}

pub fn get_mesh_or_generate(
    part_id: PartId,
    part: &Part,
    mesh_handles: &mut MeshHandles,
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