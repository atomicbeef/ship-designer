use bevy::prelude::*;
use ship_designer_client::fixed_input::FixedInput;

use crate::scaffolding::{ClientTest, FixedUpdate, MockInput};

mod scaffolding;

#[test]
fn fixed_input_just_pressed_set() {
    let mut app = App::client_test();

    app.mock_mouse_button_press(MouseButton::Left);

    app.fixed_update();

    assert!(
        app.world
            .get_resource::<FixedInput<MouseButton>>()
            .unwrap()
            .just_pressed(MouseButton::Left)
    );
}

#[test]
fn fixed_input_just_pressed_cleared() {
    let mut app = App::client_test();

    app.mock_mouse_button_press(MouseButton::Left);

    app.fixed_update();
    app.fixed_update();

    assert!(
        !app.world
            .get_resource::<FixedInput<MouseButton>>()
            .unwrap()
            .just_pressed(MouseButton::Left)
    );
}

#[test]
fn fixed_input_pressed_cleared() {
    let mut app = App::client_test();

    app.mock_mouse_button_press(MouseButton::Left);

    app.fixed_update();

    app.mock_mouse_button_release(MouseButton::Left);

    app.fixed_update();

    assert!(
        !app.world
            .get_resource::<FixedInput<MouseButton>>()
            .unwrap()
            .pressed(MouseButton::Left)
    );
}