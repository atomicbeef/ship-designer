use std::time::Duration;

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::render::mesh::MeshPlugin;
use bevy::scene::ScenePlugin;
use bevy_rapier3d::prelude::*;
use common::PHYSICS_TIMESTEP;
use common::missile::MissilePlugin;
use common::part::{PartPlugin, Parts};
use common::predefined_parts::add_hardcoded_parts;

use crate::missile::ServerMissilePlugin;
use crate::network_id_generator::NetworkIdGenerator;
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
        self.add_plugins(MinimalPlugins)
            .add_plugin(LogPlugin {
                level: Level::DEBUG,
                filter: String::new()
            })
            .add_plugin(TransformPlugin)
            .add_plugin(HierarchyPlugin)
            .add_plugin(DiagnosticsPlugin)
            .add_plugin(AssetPlugin::default())
            .add_plugin(MeshPlugin)
            .add_plugin(ScenePlugin)
    }
}

pub trait SetupRapier {
    fn setup_rapier(&mut self) -> &mut Self;
}

impl SetupRapier for App {
    fn setup_rapier(&mut self) -> &mut Self {
        let rapier_config = RapierConfiguration {
            timestep_mode: TimestepMode::Fixed {
                dt: PHYSICS_TIMESTEP,
                substeps: 1,
            },
            gravity: Vec3::default(),
            ..Default::default()
        };
    
        self.add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false))
            .add_systems(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                    .in_base_set(PhysicsSet::SyncBackend)
                    .in_schedule(CoreSchedule::FixedUpdate)
            )
            .add_systems(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackendFlush)
                    .in_base_set(PhysicsSet::SyncBackendFlush)
                    .in_schedule(CoreSchedule::FixedUpdate)
            )
            .add_systems(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                    .in_base_set(PhysicsSet::StepSimulation)
                    .in_schedule(CoreSchedule::FixedUpdate)
            )
            .add_systems(
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                    .in_base_set(PhysicsSet::Writeback)
                    .in_schedule(CoreSchedule::FixedUpdate)
            )
            .insert_resource(rapier_config)
    }
}

pub trait SetupServerSpecific {
    fn setup_server_specific(&mut self) -> &mut Self;
}

impl SetupServerSpecific for App {
    fn setup_server_specific(&mut self) -> &mut Self {
        self.add_plugin(PartPlugin)
            .add_plugin(ServerPartPlugin)
            .add_plugin(PlayerConnectionPlugin)
            .add_plugin(MissilePlugin)
            .add_plugin(ServerMissilePlugin)
            .insert_resource(FixedTime::new(Duration::from_secs_f32(PHYSICS_TIMESTEP)))
            .insert_resource(NetworkIdGenerator::new())
            .add_startup_system(setup_hardcoded_parts)
    }
}