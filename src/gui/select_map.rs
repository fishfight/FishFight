use fishsticks::GamepadContext;
use core::prelude::*;

use core::gui::{WINDOW_MARGIN_H, WINDOW_MARGIN_V};
use core::gui::background::draw_main_menu_background;
use crate::GuiTheme;

use crate::macroquad::ui::{root_ui, widgets};
use crate::macroquad::window::next_frame;

const MAP_SELECT_SCREEN_MARGIN_FACTOR: f32 = 0.1;
const MAP_SELECT_PREVIEW_TARGET_WIDTH: f32 = 250.0;
const MAP_SELECT_PREVIEW_RATIO: f32 = 10.0 / 16.0;
const MAP_SELECT_PREVIEW_SHRINK_FACTOR: f32 = 0.8;

pub async fn show_select_map_menu() -> MapResource {
    let mut current_page: i32;
    let mut hovered: i32 = 0;

    let mut old_mouse_position = get_mouse_position();

    // skip a frame to let Enter be unpressed from the previous screen
    next_frame().await;

    loop {
        draw_main_menu_background(false);

        let mut gamepad_ctx = storage::get_mut::<GamepadContext>();

        let _ = gamepad_ctx.update();

        let mut up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        let mut down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        let mut right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let mut left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let mut start = is_key_pressed(KeyCode::Enter);

        let (page_up, page_down) = {
            let mouse_wheel = get_mouse_wheel_values();
            (mouse_wheel.y > 0.0, mouse_wheel.y < 0.0)
        };

        for (_, gamepad) in gamepad_ctx.gamepads() {
            use fishsticks::{Axis, Button};

            up |= gamepad.digital_inputs.just_activated(Button::DPadUp)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickY),
                    Some(value) if value < 0.0
                );

            down |= gamepad.digital_inputs.just_activated(Button::DPadDown)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickY),
                    Some(value) if value > 0.0
                );

            left |= gamepad.digital_inputs.just_activated(Button::DPadLeft)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickX),
                    Some(value) if value < 0.0
                );

            right |= gamepad.digital_inputs.just_activated(Button::DPadRight)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickX),
                    Some(value) if value > 0.0
                );

            start |= gamepad.digital_inputs.just_activated(Button::South)
                || gamepad.digital_inputs.just_activated(Button::Start);
        }

        let map_cnt = iter_maps().len();

        let gui_theme = storage::get::<GuiTheme>();
        root_ui().push_skin(&gui_theme.map_selection);

        let viewport = get_viewport();
        let screen_margins = vec2(
            viewport.width * MAP_SELECT_SCREEN_MARGIN_FACTOR,
            viewport.height * MAP_SELECT_SCREEN_MARGIN_FACTOR,
        );
        let content_size = vec2(
            viewport.width - (screen_margins.x * 2.0),
            viewport.height - (screen_margins.y * 2.0),
        );

        let entries_per_row = (content_size.x / MAP_SELECT_PREVIEW_TARGET_WIDTH).round() as usize;
        let row_cnt = (map_cnt / entries_per_row) + 1;

        let entry_size = {
            let width = content_size.x / entries_per_row as f32;
            vec2(width, width * MAP_SELECT_PREVIEW_RATIO)
        };

        let rows_per_page = (content_size.y / entry_size.y) as usize;
        let entries_per_page = rows_per_page * entries_per_row;

        let page_cnt = (row_cnt / rows_per_page) + 1;

        {
            if up {
                hovered -= entries_per_row as i32;
                if hovered < 0 {
                    hovered += 1 + map_cnt as i32 + (map_cnt % entries_per_row) as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if down {
                let old = hovered;
                hovered += entries_per_row as i32;
                if hovered >= map_cnt as i32 {
                    if old == map_cnt as i32 - 1 {
                        hovered = 0;
                    } else {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if left {
                let row_begin = (hovered / entries_per_row as i32) * entries_per_row as i32;
                hovered -= 1;
                if hovered < row_begin {
                    hovered = row_begin + entries_per_row as i32 - 1;
                }
            }

            if right {
                let row_begin = (hovered / entries_per_row as i32) * entries_per_row as i32;
                hovered += 1;
                if hovered >= row_begin + entries_per_row as i32 {
                    hovered = row_begin;
                }
            }

            current_page = hovered / entries_per_page as i32;

            if page_up {
                current_page -= 1;
                if current_page < 0 {
                    current_page = page_cnt as i32 - 1;
                    hovered += (map_cnt + (entries_per_page - (map_cnt % entries_per_page))
                        - entries_per_page) as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1
                    }
                } else {
                    hovered -= entries_per_page as i32;
                }
            }

            if page_down {
                current_page += 1;
                if current_page >= page_cnt as i32 {
                    current_page = 0;
                    hovered %= entries_per_page as i32;
                } else {
                    hovered += entries_per_page as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            current_page %= page_cnt as i32;

            {
                if page_cnt > 1 {
                    let pagination_label = format!("page {}/{}", current_page + 1, page_cnt);

                    let label_size = root_ui().calc_size(&pagination_label);
                    let label_position =
                        viewport.as_vec2() - vec2(WINDOW_MARGIN_H, WINDOW_MARGIN_V) - label_size;

                    widgets::Label::new(&pagination_label)
                        .position(label_position)
                        .ui(&mut *root_ui());
                }

                let begin = (current_page as usize * entries_per_page).clamp(0, map_cnt);
                let end = (begin as usize + entries_per_page).clamp(begin, map_cnt);

                for (pi, i) in (begin..end).enumerate() {
                    let map_entry = get_map(i);
                    let is_hovered = hovered == i as i32;

                    let mut rect = Rect::new(
                        screen_margins.x + ((pi % entries_per_row) as f32 * entry_size.x),
                        screen_margins.y + ((pi / entries_per_row) as f32 * entry_size.y),
                        entry_size.x,
                        entry_size.y,
                    );

                    if !is_hovered {
                        let w = rect.w * MAP_SELECT_PREVIEW_SHRINK_FACTOR;
                        let h = rect.h * MAP_SELECT_PREVIEW_SHRINK_FACTOR;

                        rect.x += (rect.w - w) / 2.0;
                        rect.y += (rect.h - h) / 2.0;

                        rect.w = w;
                        rect.h = h;
                    }

                    let mouse_position = get_mouse_position();

                    if old_mouse_position != mouse_position
                        && rect.contains(mouse_position.into())
                    {
                        hovered = i as _;
                    }

                    let texture: core::macroquad::texture::Texture2D = map_entry.preview.into();

                    if widgets::Button::new(texture)
                        .size(rect.size())
                        .position(rect.point())
                        .ui(&mut *root_ui())
                        || start
                    {
                        root_ui().pop_skin();
                        let res = get_map(hovered as usize).clone();
                        return res;
                    }
                }
            }
        }

        root_ui().pop_skin();

        old_mouse_position = get_mouse_position();

        next_frame().await;
    }
}
