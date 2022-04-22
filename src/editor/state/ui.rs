use crate::editor::input::EditorInput;

mod level;
mod side_panel;
mod toolbar;
mod windows;

impl super::Editor {
    pub fn ui(&mut self, egui_ctx: &egui::Context) {
        self.draw_toolbar(egui_ctx);
        self.draw_side_panel(egui_ctx);
        self.draw_level(egui_ctx);
        self.draw_windows(egui_ctx);
    }
}
