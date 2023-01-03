use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use common::shape::{Shape, ShapeHandle, Shapes, VOXEL_SIZE};
use common::materials::Material;

use crate::meshes::MeshHandles;

pub fn generate_shape_mesh(
    shape: &Shape
) -> Mesh {
    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    for s_x in 0..shape.width() {
        for s_y in 0..shape.height() {
            for s_z in 0..shape.depth() {
                if matches!(shape.get(s_x, s_y, s_z), Material::Empty) {
                    continue;
                }

                let x = s_x as f32 * VOXEL_SIZE;
                let y = s_y as f32 * VOXEL_SIZE;
                let z = s_z as f32 * VOXEL_SIZE;
                let vertex_index_offset = vertices.len() as u32;

                // Front vertices
                vertices.push([x, y, -z]);
                vertices.push([x, y + VOXEL_SIZE, -z]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z]);
                vertices.push([x + VOXEL_SIZE, y, -z]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);

                // Front triangles
                triangles.extend([vertex_index_offset + 2, vertex_index_offset + 1, vertex_index_offset]);
                triangles.extend([vertex_index_offset, vertex_index_offset + 3, vertex_index_offset + 2]);

                // Left vertices
                vertices.push([x, y, -z]);
                vertices.push([x, y, -z - VOXEL_SIZE]);
                vertices.push([x, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x, y + VOXEL_SIZE, -z]);
                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);
                normals.push([-1.0, 0.0, 0.0]);

                // Left triangles
                triangles.extend([vertex_index_offset + 5, vertex_index_offset + 4, vertex_index_offset + 7]);
                triangles.extend([vertex_index_offset + 7, vertex_index_offset + 6, vertex_index_offset + 5]);

                // Top vertices
                vertices.push([x, y + VOXEL_SIZE, -z]);
                vertices.push([x, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z]);
                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                normals.push([0.0, 1.0, 0.0]);

                // Top faces
                triangles.extend([vertex_index_offset + 9, vertex_index_offset + 8, vertex_index_offset + 11]);
                triangles.extend([vertex_index_offset + 11, vertex_index_offset + 10, vertex_index_offset + 9]);

                // Back vertices
                vertices.push([x, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x, y, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y, -z - VOXEL_SIZE]);
                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);
                normals.push([0.0, 0.0, -1.0]);

                // Back faces
                triangles.extend([vertex_index_offset + 15, vertex_index_offset + 13, vertex_index_offset + 12]);
                triangles.extend([vertex_index_offset + 12, vertex_index_offset + 14, vertex_index_offset + 15]);

                // Right vertices
                vertices.push([x + VOXEL_SIZE, y, -z]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z]);
                vertices.push([x + VOXEL_SIZE, y + VOXEL_SIZE, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y, -z - VOXEL_SIZE]);
                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);
                normals.push([1.0, 0.0, 0.0]);

                // Right faces
                triangles.extend([vertex_index_offset + 17, vertex_index_offset + 16, vertex_index_offset + 19]);
                triangles.extend([vertex_index_offset + 19, vertex_index_offset + 18, vertex_index_offset + 17]);

                // Bottom vertices
                vertices.push([x, y, -z]);
                vertices.push([x, y, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y, -z - VOXEL_SIZE]);
                vertices.push([x + VOXEL_SIZE, y, -z]);
                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);
                normals.push([0.0, -1.0, 0.0]);

                // Bottom faces
                triangles.extend([vertex_index_offset + 20, vertex_index_offset + 21, vertex_index_offset + 22]);
                triangles.extend([vertex_index_offset + 22, vertex_index_offset + 23, vertex_index_offset + 20]);
            }
        }
    }

    // Center mesh to align with colliders
    for vertex in vertices.iter_mut() {
        vertex[0] -= shape.width() as f32 * VOXEL_SIZE / 2.0;
        vertex[1] -= shape.height() as f32 * VOXEL_SIZE / 2.0;
        vertex[2] += shape.depth() as f32 * VOXEL_SIZE / 2.0;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U32(triangles)));

    mesh
}

pub struct RegenerateShapeMesh(pub Entity);

pub fn regenerate_shape_mesh(
    mut regenerate_shape_mesh_reader: EventReader<RegenerateShapeMesh>,
    shape_handle_query: Query<&ShapeHandle>,
    shapes: Res<Shapes>,
    mut mesh_handles: ResMut<MeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands
) {
    for request in regenerate_shape_mesh_reader.iter() {
        if let Ok(shape_handle) = shape_handle_query.get(request.0) {
            if let Some(shape) = shapes.get(shape_handle) {
                let mesh = generate_shape_mesh(shape);
                let mesh_handle = meshes.add(mesh);

                mesh_handles.update(shape_handle.clone(), mesh_handle.clone());
                
                commands.entity(request.0).remove::<Handle<Mesh>>();
                commands.entity(request.0).insert(mesh_handle);
            }
        }
    }
}