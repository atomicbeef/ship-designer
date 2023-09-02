use bevy::prelude::*;
use common::fixed_update::FixedUpdateSet;
use common::part::Parts;
use common::predefined_parts::add_hardcoded_parts;
use common::{part::PartPlugin, missile::MissilePlugin};

use crate::camera::CameraPlugin;
use crate::packet_handling::process_packets;
use crate::part::meshes::PartMeshHandles;
use crate::part::meshes::mesh_generation::generate_part_mesh;
use crate::settings;
use crate::fixed_input::FixedInputPlugin;
use crate::free_camera::FreeCameraPlugin;
use crate::building::BuildingPlugin;
use crate::player_connection::PlayerConnectionPlugin;
use crate::part::ClientPartPlugin;
use crate::player_controller::PlayerControllerPlugin;
use crate::missile::ClientMissilePlugin;

pub fn setup_hardcoded_parts(
    mut parts: ResMut<Parts>,
    mut mesh_handles: ResMut<PartMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    let part_handles = add_hardcoded_parts(&mut parts);

    for part_handle in part_handles {
        let part = parts.get(&part_handle).unwrap();
        let mesh = generate_part_mesh(part);
        let mesh_handle = meshes.add(mesh);
        mesh_handles.add(part_handle.id(), mesh_handle);
    }
}

pub trait SetupClientSpecific {
    fn setup_client_specific(&mut self) -> &mut Self;
}

impl SetupClientSpecific for App {
    fn setup_client_specific(&mut self) -> &mut Self {
        self.insert_resource(settings::Settings::default())
            .add_plugins((
                FixedInputPlugin,
                CameraPlugin,
                FreeCameraPlugin,
                BuildingPlugin,
                PlayerConnectionPlugin,
                PartPlugin,
                ClientPartPlugin,
                PlayerControllerPlugin,
                MissilePlugin,
                ClientMissilePlugin,
            ))
            .add_systems(FixedUpdate, process_packets.in_set(FixedUpdateSet::PreUpdate))
            .add_systems(Startup, setup_hardcoded_parts)
    }
}