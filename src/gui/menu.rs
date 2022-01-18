use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Id, Ui},
};

use fishsticks::{Axis, Button};

use super::{
    GuiResources, Panel, BUTTON_FONT_SIZE, BUTTON_MARGIN_V, WINDOW_MARGIN_H, WINDOW_MARGIN_V,
};

use crate::{is_gamepad_btn_pressed, GamepadContext};

#[derive(Debug, Copy, Clone)]
pub enum MenuPosition {
    Center,
    AbsoluteHorizontal(f32),
    AbsoluteVertical(f32),
    Absolute(Vec2),
}

impl Default for MenuPosition {
    fn default() -> Self {
        MenuPosition::Center
    }
}

impl From<(Option<f32>, Option<f32>)> for MenuPosition {
    fn from((x, y): (Option<f32>, Option<f32>)) -> Self {
        let mut res = Self::Center;

        if let Some(x) = x {
            res = Self::AbsoluteHorizontal(x);
        }

        if let Some(y) = y {
            if let Self::AbsoluteHorizontal(x) = res {
                res = Self::Absolute(vec2(x, y));
            } else {
                res = Self::AbsoluteVertical(y);
            }
        }

        res
    }
}

impl From<Vec2> for MenuPosition {
    fn from(position: Vec2) -> Self {
        Self::Absolute(position)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MenuEntry {
    pub index: usize,
    pub title: String,
    pub is_pulled_down: bool,
    pub is_disabled: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct MenuResult(usize);

impl MenuResult {
    pub fn is_cancel(self) -> bool {
        self.0 == Menu::CANCEL_INDEX
    }

    pub fn into_usize(self) -> usize {
        self.into()
    }
}

impl From<usize> for MenuResult {
    fn from(index: usize) -> Self {
        MenuResult(index)
    }
}

impl From<MenuResult> for usize {
    fn from(res: MenuResult) -> Self {
        res.0
    }
}

pub struct Menu {
    id: Id,
    header: Option<String>,
    width: f32,
    height: Option<f32>,
    position: MenuPosition,
    entries: Vec<MenuEntry>,
    has_cancel_button: bool,
    cancel_entry_title_override: Option<String>,
    current_selection: Option<usize>,
    last_mouse_position: Vec2,
    up_grace_timer: f32,
    down_grace_timer: f32,
    is_first_draw: bool,
}

impl Menu {
    pub const CANCEL_INDEX: usize = 99999;

    const HEADER_MARGIN: f32 = 16.0;
    const CANCEL_TITLE: &'static str = "Cancel";

    pub const ENTRY_HEIGHT: f32 = (BUTTON_MARGIN_V * 2.0) + BUTTON_FONT_SIZE;
    pub const ENTRY_MARGIN: f32 = 4.0;

    const NAVIGATION_GRACE_TIME: f32 = 0.25;

    pub fn new(id: Id, width: f32, entries: &[MenuEntry]) -> Self {
        Menu {
            id,
            header: None,
            width,
            height: None,
            position: MenuPosition::default(),
            entries: entries.to_vec(),
            has_cancel_button: false,
            cancel_entry_title_override: None,
            current_selection: None,
            last_mouse_position: Vec2::ZERO,
            up_grace_timer: Self::NAVIGATION_GRACE_TIME,
            down_grace_timer: Self::NAVIGATION_GRACE_TIME,
            is_first_draw: true,
        }
    }

    #[allow(dead_code)]
    pub fn with_header(self, header: &str) -> Self {
        let header = Some(header.to_string());

        Menu { header, ..self }
    }

    #[allow(dead_code)]
    pub fn with_height(self, height: f32) -> Self {
        Menu {
            height: Some(height),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn with_position<P: Into<MenuPosition>>(self, position: P) -> Self {
        Menu {
            position: position.into(),
            ..self
        }
    }

    /// This adds a cancel entry to the menu. The title of this entry will be `Self::CANCEL_TITLE`
    /// if no override is specified
    pub fn with_cancel_button(self, title_override: Option<&str>) -> Self {
        for entry in &self.entries {
            assert_ne!(
                entry.index,
                Self::CANCEL_INDEX,
                "Menu: MenuEntry has reserved index ({})",
                Self::CANCEL_INDEX
            );
        }

        let cancel_entry_title_override = title_override.map(|str| str.to_string());

        Menu {
            has_cancel_button: true,
            cancel_entry_title_override,
            ..self
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Option<MenuResult> {
        let mut res = None;

        {
            let dt = get_frame_time();
            self.up_grace_timer += dt;
            self.down_grace_timer += dt;
        }

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.menu);
        }

        let mouse_position = {
            let (x, y) = mouse_position();
            vec2(x, y)
        };

        if mouse_position != self.last_mouse_position {
            self.current_selection = None;
        }

        self.last_mouse_position = mouse_position;

        let mut entries = self.entries.clone();

        if self.has_cancel_button {
            let title = self
                .cancel_entry_title_override
                .clone()
                .unwrap_or_else(|| Self::CANCEL_TITLE.to_string());

            entries.push(MenuEntry {
                index: Self::CANCEL_INDEX,
                title,
                is_pulled_down: true,
                is_disabled: false,
            })
        }

        let (should_confirm, should_cancel) = self.update_input();

        let header_height = if let Some(header) = &self.header {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.menu_header);

            let header_size = ui.calc_size(header);

            ui.pop_skin();

            header_size.y + Self::HEADER_MARGIN
        } else {
            0.0
        };

        let size = {
            let height = header_height
                + if let Some(height) = self.height {
                    height
                } else {
                    let len = entries.len();
                    let entry_margins = if len > 0 {
                        (len as f32 * Self::ENTRY_MARGIN) - Self::ENTRY_MARGIN
                    } else {
                        0.0
                    };

                    (len as f32 * Self::ENTRY_HEIGHT) + entry_margins + (WINDOW_MARGIN_V * 2.0)
                };

            vec2(self.width, height)
        };

        let position = match self.position {
            MenuPosition::Center => vec2(screen_width() - size.x, screen_height() - size.y) / 2.0,
            MenuPosition::AbsoluteHorizontal(x) => vec2(x, (screen_height() - size.y) / 2.0),
            MenuPosition::AbsoluteVertical(y) => vec2((screen_width() - size.x) / 2.0, y),
            MenuPosition::Absolute(position) => position,
        };

        Panel::new(self.id, size, position).ui(ui, |ui, inner_size| {
            let entry_size = vec2(size.x - (WINDOW_MARGIN_H * 2.0), Self::ENTRY_HEIGHT);

            if let Some(header) = &self.header {
                let gui_resources = storage::get::<GuiResources>();

                ui.push_skin(&gui_resources.skins.menu_header);

                ui.label(vec2(0.0, 0.0), header);

                ui.pop_skin();
            }

            let entries_position = vec2(0.0, header_height);

            let mut top_entries = Vec::new();
            let mut bottom_entries = Vec::new();

            for entry in entries {
                if entry.is_pulled_down {
                    bottom_entries.push(entry);
                } else {
                    top_entries.push(entry);
                }
            }

            if should_confirm && self.current_selection.is_none() {
                let mut entry = top_entries.first();

                if entry.is_none() {
                    entry = bottom_entries.first();
                }

                if let Some(entry) = entry {
                    res = Some(entry.index.into());
                }
            }

            for (i, entry) in top_entries.iter().enumerate() {
                let entry_position = entries_position
                    + if i > 0 {
                        vec2(0.0, i as f32 * (entry_size.y + Self::ENTRY_MARGIN))
                    } else {
                        vec2(0.0, 0.0)
                    };

                let is_selected = if let Some(current_selection) = self.current_selection {
                    current_selection == i
                } else {
                    false
                };

                {
                    let gui_resources = storage::get::<GuiResources>();
                    if entry.is_disabled {
                        ui.push_skin(&gui_resources.skins.menu_disabled);
                    } else if is_selected {
                        ui.push_skin(&gui_resources.skins.menu_selected);
                    }
                }

                let btn = widgets::Button::new(entry.title.as_str())
                    .size(entry_size)
                    .position(entry_position);

                if (btn.ui(ui) || (is_selected && should_confirm)) && !entry.is_disabled {
                    res = Some(entry.index.into());
                }

                if entry.is_disabled || is_selected {
                    ui.pop_skin();
                }
            }

            let bottom_y = {
                let top_end = entries_position.y
                    + (top_entries.len() as f32 * (entry_size.y + Self::ENTRY_MARGIN));
                let bottom_height = {
                    let len = bottom_entries.len();

                    let entry_margins = if len > 0 {
                        (len as f32 * Self::ENTRY_MARGIN) - Self::ENTRY_MARGIN
                    } else {
                        0.0
                    };

                    (len as f32 * entry_size.y) + entry_margins
                };

                if top_end + bottom_height > inner_size.y {
                    top_end
                } else {
                    inner_size.y - bottom_height
                }
            };

            for (i, entry) in bottom_entries.iter().enumerate() {
                let entry_position = vec2(
                    0.0,
                    bottom_y + (i as f32 * (entry_size.y + Self::ENTRY_MARGIN)),
                );

                let mut is_selected = false;
                if let Some(current_selection) = self.current_selection {
                    if current_selection >= top_entries.len() {
                        is_selected = current_selection - top_entries.len() == i
                    }
                }

                {
                    let gui_resources = storage::get::<GuiResources>();
                    if entry.is_disabled {
                        ui.push_skin(&gui_resources.skins.menu_disabled);
                    } else if is_selected {
                        ui.push_skin(&gui_resources.skins.menu_selected);
                    }
                }

                let btn = widgets::Button::new(entry.title.as_str())
                    .size(entry_size)
                    .position(entry_position);

                if (btn.ui(ui) || (is_selected && should_confirm)) && !entry.is_disabled {
                    res = Some(entry.index.into());
                }

                if entry.is_disabled || is_selected {
                    ui.pop_skin();
                }
            }
        });

        ui.pop_skin();

        if self.has_cancel_button && should_cancel {
            res = Some(Self::CANCEL_INDEX.into());
        }

        self.is_first_draw = false;

        res
    }

    fn update_input(&mut self) -> (bool, bool) {
        if self.is_first_draw {
            (false, false)
        } else {
            let gamepad_context = storage::get::<GamepadContext>();

            let mut selection = self.current_selection.map(|s| s as i32);

            let mut gamepad_up = false;
            let mut gamepad_down = false;

            for (_, gamepad) in gamepad_context.gamepads() {
                gamepad_up = gamepad_up
                    || gamepad.digital_inputs.activated(Button::DPadUp)
                    || gamepad.analog_inputs.digital_value(Axis::LeftStickY) < 0.0;
                gamepad_down = gamepad_down
                    || gamepad.digital_inputs.activated(Button::DPadDown)
                    || gamepad.analog_inputs.digital_value(Axis::LeftStickY) > 0.0;
            }

            if self.up_grace_timer >= Self::NAVIGATION_GRACE_TIME
                && (gamepad_up || is_key_down(KeyCode::Up) || is_key_down(KeyCode::W))
            {
                self.up_grace_timer = 0.0;

                selection = if let Some(selection) = selection {
                    Some(selection - 1)
                } else {
                    Some(0)
                };
            } else if self.down_grace_timer >= Self::NAVIGATION_GRACE_TIME
                && (gamepad_down || is_key_down(KeyCode::Down) || is_key_down(KeyCode::S))
            {
                self.down_grace_timer = 0.0;

                selection = if let Some(selection) = selection {
                    Some(selection + 1)
                } else {
                    Some(0)
                };
            }

            if let Some(selection) = selection {
                let mut entries_cnt = self.entries.len();
                if self.has_cancel_button {
                    entries_cnt += 1;
                }

                let selection = if selection < 0 {
                    if entries_cnt > 0 {
                        entries_cnt - 1
                    } else {
                        0
                    }
                } else {
                    selection as usize % entries_cnt
                };

                self.current_selection = Some(selection);
            }

            let should_confirm = is_gamepad_btn_pressed(Some(&gamepad_context), Button::South)
                || is_key_pressed(KeyCode::Enter);

            let should_cancel = is_gamepad_btn_pressed(Some(&gamepad_context), Button::East)
                || is_key_pressed(KeyCode::Escape);

            (should_confirm, should_cancel)
        }
    }
}
