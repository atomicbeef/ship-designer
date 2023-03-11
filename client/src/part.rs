use bevy::prelude::*;

use common::events::part::UpdateVoxels;
use common::index::Index;
use common::network_id::NetworkId;
use common::part::{PartHandle, Parts};

use crate::RegeneratePartMesh;

pub fn update_voxels(
    mut update_voxels_reader: EventReader<UpdateVoxels>,
    mut regenerate_part_mesh_writer: EventWriter<RegeneratePartMesh>,
    network_id_index: Res<Index<NetworkId>>,
    mut part_handle_query: Query<&mut PartHandle>,
    mut parts: ResMut<Parts>
) {
    for voxel_update in update_voxels_reader.iter() {
        if let Some(entity) = network_id_index.entity(&voxel_update.network_id) {
            if let Ok(mut part_handle) = part_handle_query.get_mut(entity) {
                if let Some(part) = parts.get_mut(&mut part_handle) {
                    part.set_voxels(&voxel_update.voxels);
                    regenerate_part_mesh_writer.send(RegeneratePartMesh(entity));
                }
            }
        }
    }
}