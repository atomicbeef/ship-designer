use bevy::prelude::*;

use common::events::shape::UpdateVoxels;
use common::index::Index;
use common::network_id::NetworkId;
use common::shape::{ShapeHandle, Shapes};

use crate::RegenerateShapeMesh;

pub fn update_voxels(
    mut update_voxels_reader: EventReader<UpdateVoxels>,
    mut regenerate_shape_mesh_writer: EventWriter<RegenerateShapeMesh>,
    network_id_index: Res<Index<NetworkId>>,
    mut shape_handle_query: Query<&mut ShapeHandle>,
    mut shapes: ResMut<Shapes>
) {
    for voxel_update in update_voxels_reader.iter() {
        if let Some(entity) = network_id_index.entity(&voxel_update.network_id) {
            if let Ok(mut shape_handle) = shape_handle_query.get_mut(entity) {
                if let Some(shape) = shapes.get_mut(&mut shape_handle) {
                    shape.set_voxels(&voxel_update.voxels);
                    regenerate_shape_mesh_writer.send(RegenerateShapeMesh(entity));
                }
            }
        }
    }
}