use std::marker::PhantomData;

use bevy::{prelude::*, transform::{systems::{sync_simple_transforms, propagate_transforms}, TransformSystem}};
use bevy_rapier3d::prelude::*;

use crate::PHYSICS_TIMESTEP;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
#[system_set(base)]
pub enum FixedUpdateSet {
    PreUpdate,
    PreUpdateFlush,
    Update,
    UpdateFlush,
    PostUpdate,
    PostUpdateFlush,
    Last,
    LastFlush,
}

pub struct NetworkSendSet;

// A set for `propagate_transforms` to mark it as ambiguous with `sync_simple_transforms`.
// Used instead of the `SystemTypeSet` as that would not allow multiple instances of the system.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct PropagateTransformsSet;

pub trait SetupFixedTimeStepSchedule {
    fn setup_fixed_timestep_schedule(&mut self) -> &mut Self;
}

impl SetupFixedTimeStepSchedule for App {
    fn setup_fixed_timestep_schedule(&mut self) -> &mut Self {
        self.edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
            schedule.configure_sets((
                FixedUpdateSet::PreUpdate,
                FixedUpdateSet::PreUpdateFlush,
                FixedUpdateSet::Update,
                FixedUpdateSet::UpdateFlush,
                PhysicsSet::SyncBackend,
                PhysicsSet::SyncBackendFlush,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
                FixedUpdateSet::PostUpdate,
                FixedUpdateSet::PostUpdateFlush,
                FixedUpdateSet::Last,
                FixedUpdateSet::LastFlush,
            ).chain());

            schedule.configure_set(TransformSystem::TransformPropagate.in_base_set(FixedUpdateSet::PostUpdate));
            schedule.configure_set(PropagateTransformsSet.in_set(TransformSystem::TransformPropagate));
    
            schedule.set_default_base_set(FixedUpdateSet::Update);
    
            schedule.add_system(apply_system_buffers.in_base_set(FixedUpdateSet::PreUpdateFlush));
            schedule.add_system(apply_system_buffers.in_base_set(FixedUpdateSet::UpdateFlush));
            schedule.add_system(apply_system_buffers.in_base_set(FixedUpdateSet::PostUpdateFlush));
            schedule.add_system(apply_system_buffers.in_base_set(FixedUpdateSet::LastFlush));

            schedule.add_system(sync_simple_transforms.in_set(TransformSystem::TransformPropagate)
                .ambiguous_with(PropagateTransformsSet)
            );
            schedule.add_system(propagate_transforms.in_set(PropagateTransformsSet));
        })
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

#[derive(Resource)]
pub struct Flag<T> {
    pub enabled: bool,
    _marker: PhantomData<T>,
}

impl<T> Default for Flag<T> {
    fn default() -> Self {
        Self {
            enabled: false,
            _marker: PhantomData,
        }
    }
}

fn set_update_events<T: Event>(mut update_events_flag: ResMut<Flag<Events<T>>>) {
    update_events_flag.enabled = true;
}

fn update_events<T: Event>(mut update_events_flag: ResMut<Flag<Events<T>>>, mut events: ResMut<Events<T>>) {
    if update_events_flag.enabled {
        events.update();
        update_events_flag.enabled = false;
    }
}

pub trait AddFixedEvent {
    fn add_fixed_event<T: Event>(&mut self) -> &mut Self;
}

impl AddFixedEvent for App {
    fn add_fixed_event<T: Event>(&mut self) -> &mut Self {
        self.init_resource::<Flag<Events<T>>>()
            .init_resource::<Events<T>>()
            .add_systems((
                set_update_events::<T>.in_base_set(FixedUpdateSet::PreUpdate),
                update_events::<T>.in_base_set(FixedUpdateSet::Last),
            ).in_schedule(CoreSchedule::FixedUpdate))
    }
}