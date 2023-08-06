use std::time::Duration;

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use common::PHYSICS_TIMESTEP;
use common::fixed_update::FixedUpdateSet;
use common::missile::MissilePlugin;
use common::part::{PartPlugin, Parts};
use common::predefined_parts::add_hardcoded_parts;

use crate::missile::ServerMissilePlugin;
use crate::network_id_generator::NetworkIdGenerator;
use crate::packet_handling::process_packets;
use crate::part::ServerPartPlugin;
use crate::player_connection::PlayerConnectionPlugin;

pub fn setup_hardcoded_parts(mut parts: ResMut<Parts>) {
    add_hardcoded_parts(&mut parts);
}

pub trait SetupBevyPlugins {
    fn setup_bevy_plugins(&mut self) -> &mut Self;
}

impl SetupBevyPlugins for App {
    fn setup_bevy_plugins(&mut self) -> &mut Self {
        self.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            HierarchyPlugin,
            DiagnosticsPlugin,
            AssetPlugin::default(),
            MeshPlugin,
            ScenePlugin,
        ))
    }
}

pub trait SetupServerSpecific {
    fn setup_server_specific(&mut self) -> &mut Self;
}

impl SetupServerSpecific for App {
    fn setup_server_specific(&mut self) -> &mut Self {
        self.add_plugins((
                PartPlugin,
                ServerPartPlugin,
                PlayerConnectionPlugin,
                MissilePlugin,
                ServerMissilePlugin,
            ))
            .insert_resource(FixedTime::new(Duration::from_secs_f32(PHYSICS_TIMESTEP)))
            .insert_resource(NetworkIdGenerator::new())
            .add_systems(Startup, setup_hardcoded_parts)
            .add_systems(FixedUpdate, process_packets.in_set(FixedUpdateSet::PreUpdate))
    }
}