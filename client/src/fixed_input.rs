use std::hash::Hash;

use bevy::{prelude::*, input::{InputSystem, mouse::MouseMotion}};

use common::fixed_update::{Flag, FixedUpdateSet, AddFixedEvent};

#[derive(Debug, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct FixedInput<T: Copy + Eq + Hash + Send + Sync + 'static>(Input<T>);

impl<T: Copy + Eq + Hash + Send + Sync + 'static> Default for FixedInput<T> {
    fn default() -> Self {
        Self(Input::default())
    }
}

fn update_fixed_input<T: Copy + Eq + Hash + Send + Sync + 'static>(
    mut fixed_input: ResMut<FixedInput<T>>,
    input: Res<Input<T>>
) {
    for pressed in input.get_just_pressed() {
        fixed_input.press(*pressed);
    }

    for released in input.get_just_released() {
        fixed_input.release(*released);
    }
}

fn set_clear_fixed_input<T: Copy + Eq + Hash + Send + Sync + 'static>(
    mut fixed_input: ResMut<FixedInput<T>>,
    mut flag: ResMut<Flag<Input<T>>>
) {
    if flag.enabled {
        fixed_input.clear();
    }

    flag.enabled = true;
}

fn clear_fixed_input<T: Copy + Eq + Hash + Send + Sync + 'static>(
    mut fixed_input: ResMut<FixedInput<T>>,
    mut flag: ResMut<Flag<Input<T>>>
) {
    if flag.enabled {
        fixed_input.clear();
    }

    flag.enabled = false;
}

#[derive(Debug, Clone, Resource, Reflect, Deref, DerefMut)]
pub struct FixedMouseMotion(MouseMotion);

fn send_fixed_mouse_motion_events(
    mut mouse_motion_reader: EventReader<MouseMotion>,
    mut fixed_mouse_motion_wrier: EventWriter<FixedMouseMotion>
) {
    for event in mouse_motion_reader.iter() {
        fixed_mouse_motion_wrier.send(FixedMouseMotion(*event));
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FixedInputSystem;

fn add_fixed_input<T: Copy + Eq + Hash + Send + Sync + 'static>(app: &mut App) {
    app.init_resource::<Flag<Input<T>>>()
        .init_resource::<FixedInput<T>>()
        .add_system(update_fixed_input::<T>.in_base_set(CoreSet::PreUpdate).after(InputSystem))
        .add_system(set_clear_fixed_input::<T>.in_schedule(CoreSchedule::FixedUpdate)
            .in_base_set(FixedUpdateSet::PreUpdate)
            .in_set(FixedInputSystem)
        )
        .add_system(clear_fixed_input::<T>.in_base_set(CoreSet::Last));
}

pub struct FixedInputPlugin;

impl Plugin for FixedInputPlugin {
    fn build(&self, app: &mut App) {
        add_fixed_input::<KeyCode>(app);
        add_fixed_input::<ScanCode>(app);
        add_fixed_input::<MouseButton>(app);
        add_fixed_input::<GamepadButton>(app);
        app.add_fixed_event::<FixedMouseMotion>();
        app.add_system(send_fixed_mouse_motion_events.in_schedule(CoreSchedule::FixedUpdate)
            .in_base_set(FixedUpdateSet::PreUpdate)
            .in_set(FixedInputSystem)
        );
    }
}