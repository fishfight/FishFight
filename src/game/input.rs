use macroquad::experimental::collections::storage;
use macroquad::input::{is_key_down, is_key_pressed, KeyCode};

use fishsticks::{Axis, Button, GamepadContext};

use serde::{Deserialize, Serialize};

use crate::json;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameInputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(fishsticks::GamepadId),
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameInput {
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub left: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub right: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub down: bool,
    /// Will be `true` if jump was just pressed
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub jump: bool,
    /// Will be `true` if jump was just pressed and if it is held
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub float: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub pickup: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub fire: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub slide: bool,
}

pub fn collect_local_input(input_scheme: GameInputScheme) -> GameInput {
    let mut input = GameInput::default();

    match input_scheme {
        GameInputScheme::KeyboardLeft => {
            input.pickup = is_key_pressed(KeyCode::C);
            input.fire = is_key_down(KeyCode::V) || is_key_down(KeyCode::LeftControl);

            input.jump = is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Space);
            input.float = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space);
            input.left = is_key_down(KeyCode::A);
            input.down = is_key_down(KeyCode::S);
            input.right = is_key_down(KeyCode::D);

            input.slide = is_key_pressed(KeyCode::F);
        }
        GameInputScheme::KeyboardRight => {
            input.pickup = is_key_pressed(KeyCode::K);
            input.fire = is_key_down(KeyCode::L);

            input.jump = is_key_pressed(KeyCode::Up);
            input.float = is_key_down(KeyCode::Up);
            input.left = is_key_down(KeyCode::Left);
            input.down = is_key_down(KeyCode::Down);
            input.right = is_key_down(KeyCode::Right);

            input.slide = is_key_pressed(KeyCode::RightControl);
        }
        GameInputScheme::Gamepad(ix) => {
            let gamepad_context = storage::get_mut::<GamepadContext>();
            let gamepad = gamepad_context.gamepad(ix);

            if let Some(gamepad) = gamepad {
                input.pickup = gamepad.digital_inputs.just_activated(Button::West);
                input.fire = gamepad.digital_inputs.activated(Button::East);

                input.jump = gamepad.digital_inputs.just_activated(Button::South);

                input.left = gamepad.digital_inputs.activated(Button::DPadLeft)
                    || gamepad.analog_inputs.digital_value(Axis::LeftStickX) < 0.0;

                input.right = gamepad.digital_inputs.activated(Button::DPadRight)
                    || gamepad.analog_inputs.digital_value(Axis::LeftStickX) > 0.0;

                input.down = gamepad.digital_inputs.activated(Button::DPadDown)
                    || gamepad.analog_inputs.digital_value(Axis::LeftStickY) > 0.0;

                input.slide = gamepad.digital_inputs.just_activated(Button::North)
            }
        }
    }

    input
}
