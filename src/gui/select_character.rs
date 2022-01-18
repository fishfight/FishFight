use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

use fishsticks::{Axis, Button, GamepadContext};

use crate::gui::{
    draw_main_menu_background, GuiResources, Panel, BUTTON_FONT_SIZE, BUTTON_MARGIN_H,
    WINDOW_BG_COLOR,
};
use crate::input::update_gamepad_context;
use crate::player::PlayerCharacterMetadata;
use crate::{
    draw_one_animated_sprite, update_one_animated_sprite, AnimatedSprite, AnimatedSpriteMetadata,
    GameInputScheme, Resources, Transform,
};

const SECTION_WIDTH: f32 = 300.0;
const SECTION_HEIGHT: f32 = 400.0;

const SECTION_MARGIN: f32 = 16.0;

const NAVIGATION_GRACE_TIME: f32 = 0.25;

const NAVIGATION_BTN_WIDTH: f32 = 64.0;
const NAVIGATION_BTN_HEIGHT: f32 = (BUTTON_MARGIN_H * 2.0) + BUTTON_FONT_SIZE;

pub async fn show_select_characters_menu(
    player_input: &[GameInputScheme],
) -> Vec<PlayerCharacterMetadata> {
    let mut selected_params = Vec::new();

    let player_cnt = player_input.len();

    let player_characters = {
        let resources = storage::get::<Resources>();
        resources.player_characters.clone()
    };

    assert!(
        player_characters.len() >= player_cnt,
        "Character selection: There are more players than there are available player characters"
    );

    let mut current_selections = Vec::new();
    let mut navigation_grace_timers = Vec::new();
    let mut animated_sprites = Vec::new();

    for (i, character) in player_characters.iter().enumerate().take(player_cnt) {
        selected_params.push(None);

        current_selections.push(i);
        navigation_grace_timers.push(0.0);

        let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

        let (texture, frame_size) = storage::get::<Resources>()
            .textures
            .get(&meta.texture_id)
            .map(|t| (t.texture, t.frame_size()))
            .unwrap();

        let animations = meta
            .animations
            .iter()
            .cloned()
            .map(|a| a.into())
            .collect::<Vec<_>>();

        let params = meta.into();

        let sprite = AnimatedSprite::new(texture, frame_size, animations.as_slice(), params);

        animated_sprites.push(sprite);
    }

    let mut is_ready = false;

    while !is_ready {
        update_gamepad_context(None).unwrap();

        draw_main_menu_background(false);

        let section_size = vec2(SECTION_WIDTH, SECTION_HEIGHT);
        let total_size = vec2(
            ((section_size.x + SECTION_MARGIN) * player_cnt as f32) - SECTION_MARGIN,
            section_size.y,
        );

        let first_position = (vec2(screen_width(), screen_height()) - total_size) / 2.0;

        {
            let gui_resources = storage::get::<GuiResources>();
            root_ui().push_skin(&gui_resources.skins.default);
        }

        for (i, input_scheme) in player_input.iter().enumerate() {
            let section_position = vec2(
                first_position.x + ((section_size.x + SECTION_MARGIN) * i as f32),
                first_position.y,
            );

            let mut current_selection = current_selections[i] as i32;

            let mut should_navigate_left = false;
            let mut should_navigate_right = false;
            let mut should_confirm = false;

            {
                navigation_grace_timers[i] += get_frame_time();

                let can_navigate = navigation_grace_timers[i] >= NAVIGATION_GRACE_TIME;

                match *input_scheme {
                    GameInputScheme::KeyboardRight => {
                        should_navigate_left = can_navigate && is_key_down(KeyCode::Left);
                        should_navigate_right = can_navigate && is_key_down(KeyCode::Right);
                        should_confirm =
                            is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::Enter);
                    }
                    GameInputScheme::KeyboardLeft => {
                        should_navigate_left = can_navigate && is_key_down(KeyCode::A);
                        should_navigate_right = can_navigate && is_key_down(KeyCode::D);
                        should_confirm =
                            is_key_pressed(KeyCode::V) || is_key_pressed(KeyCode::LeftControl);
                    }
                    GameInputScheme::Gamepad(gamepad_id) => {
                        let gamepad_context = storage::get::<GamepadContext>();
                        let gamepad = gamepad_context.gamepad(gamepad_id);

                        if let Some(gamepad) = gamepad {
                            should_navigate_left = can_navigate
                                && (gamepad.analog_inputs.digital_value(Axis::LeftStickX) < 0.0
                                    || gamepad.digital_inputs.just_activated(Button::DPadLeft));

                            should_navigate_right = can_navigate
                                && (gamepad.analog_inputs.digital_value(Axis::LeftStickX) > 0.0
                                    || gamepad.digital_inputs.just_activated(Button::DPadRight));

                            should_confirm = gamepad.digital_inputs.just_activated(Button::South);
                        }
                    }
                }

                Panel::new(hash!("section", i), section_size, section_position)
                    .with_title(&format!("Player {}", i + 1), true)
                    .with_background_color(WINDOW_BG_COLOR)
                    .ui(&mut *root_ui(), |ui, inner_size| {
                        let animation_player = &mut animated_sprites[i];

                        update_one_animated_sprite(animation_player);

                        // TODO: Calculate scale from a fixed target size, based on ui layout
                        animation_player.scale = 2.0;

                        let animation_size = animation_player.size();
                        let animation_transform = {
                            let position = section_position
                                + vec2((section_size.x - animation_size.x) / 2.0, 100.0);
                            Transform::from(position)
                        };

                        draw_one_animated_sprite(&animation_transform, animation_player);

                        {
                            let gui_resources = storage::get::<GuiResources>();
                            ui.push_skin(&gui_resources.skins.window_header);

                            let name_label = &player_characters[current_selection as usize].name;

                            let label_size = ui.calc_size(name_label);
                            let label_position = vec2(
                                (inner_size.x - label_size.x) / 2.0,
                                inner_size.y
                                    - NAVIGATION_BTN_HEIGHT
                                    - SECTION_MARGIN
                                    - label_size.y,
                            );

                            widgets::Label::new(name_label)
                                .position(label_position)
                                .ui(ui);

                            ui.pop_skin();
                        }

                        let btn_size = vec2(NAVIGATION_BTN_WIDTH, NAVIGATION_BTN_HEIGHT);

                        let btn_section = vec2(inner_size.x / 2.0, inner_size.y - btn_size.y);

                        {
                            let btn_position = vec2(
                                btn_section.x - (SECTION_MARGIN / 2.0) - btn_size.x,
                                btn_section.y,
                            );

                            should_navigate_left = widgets::Button::new("<")
                                .size(btn_size)
                                .position(btn_position)
                                .ui(ui)
                                || should_navigate_left;
                        }

                        {
                            let btn_position =
                                vec2(btn_section.x + (SECTION_MARGIN / 2.0), btn_section.y);

                            should_navigate_right = widgets::Button::new(">")
                                .size(btn_size)
                                .position(btn_position)
                                .ui(ui)
                                || should_navigate_right;
                        }
                    });

                if should_confirm {
                    let params = player_characters[current_selection as usize].clone();
                    selected_params[i] = Some(params);
                }
            }

            if selected_params[i].is_none() && (should_navigate_left || should_navigate_right) {
                let mut is_taken = true;
                while is_taken {
                    if should_navigate_left {
                        current_selection -= 1;
                    } else if should_navigate_right {
                        current_selection += 1;
                    }

                    if current_selection < 0 {
                        current_selection = player_characters.len() as i32 - 1;
                    } else {
                        current_selection %= player_characters.len() as i32;
                    }

                    is_taken = current_selections
                        .iter()
                        .enumerate()
                        .any(|(ii, selection)| ii != i && *selection == current_selection as usize);
                }

                current_selections[i] = current_selection as usize;

                navigation_grace_timers[i] = 0.0;

                let character = player_characters.get(current_selection as usize).unwrap();

                let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

                let (texture, frame_size) = storage::get::<Resources>()
                    .textures
                    .get(&meta.texture_id)
                    .map(|t| (t.texture, t.frame_size()))
                    .unwrap();

                let animations = meta
                    .animations
                    .iter()
                    .cloned()
                    .map(|a| a.into())
                    .collect::<Vec<_>>();

                let params = meta.into();

                animated_sprites[i] =
                    AnimatedSprite::new(texture, frame_size, animations.as_slice(), params);
            }

            is_ready = !selected_params.iter().any(|params| params.is_none());
        }

        root_ui().pop_skin();

        next_frame().await;
    }

    selected_params.into_iter().flatten().collect()
}
