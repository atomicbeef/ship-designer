use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::transform::{systems::{sync_simple_transforms, propagate_transforms}, TransformSystem};
use bevy_rapier3d::prelude::*;

use crate::PHYSICS_TIMESTEP;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
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
        self.edit_schedule(FixedUpdate, |schedule| {
            schedule.configure_sets((
                FixedUpdateSet::PreUpdate,
                FixedUpdateSet::PreUpdateFlush,
                FixedUpdateSet::Update,
                FixedUpdateSet::UpdateFlush,
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
                FixedUpdateSet::PostUpdate,
                FixedUpdateSet::PostUpdateFlush,
                FixedUpdateSet::Last,
                FixedUpdateSet::LastFlush,
            ).chain());

            schedule.configure_set(TransformSystem::TransformPropagate.in_set(FixedUpdateSet::PostUpdate));
            schedule.configure_set(PropagateTransformsSet.in_set(TransformSystem::TransformPropagate));

            schedule.add_systems(apply_deferred.in_set(FixedUpdateSet::PreUpdateFlush));
            schedule.add_systems(apply_deferred.in_set(FixedUpdateSet::UpdateFlush));
            schedule.add_systems(apply_deferred.in_set(FixedUpdateSet::PostUpdateFlush));
            schedule.add_systems(apply_deferred.in_set(FixedUpdateSet::LastFlush));

            schedule.add_systems(sync_simple_transforms.in_set(TransformSystem::TransformPropagate)
                .ambiguous_with(PropagateTransformsSet)
            );
            schedule.add_systems(propagate_transforms.in_set(PropagateTransformsSet));
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
    
        self.insert_resource(rapier_config)
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule())
            
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
            .add_systems(FixedUpdate, (
                set_update_events::<T>.in_set(FixedUpdateSet::PreUpdate),
                update_events::<T>.in_set(FixedUpdateSet::Last),
            ))
    }
}