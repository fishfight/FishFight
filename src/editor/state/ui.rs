mod level;
mod side_panel;
mod toolbar;
mod windows;

impl super::Editor {
    pub fn ui(&mut self, egui_ctx: &egui::Context) {
        // +---------+----------------------------+------------+
        // |         |                            |            |
        // | toolbar |         level_view         | side_panel |
        // |         |                            |            |
        // |         |              +--------+    |            |
        // |         |              | window +    |            |
        // |         |              +--------+    |            |
        // +---------+----------------------------+------------+

        self.draw_toolbar(egui_ctx);
        self.draw_side_panel(egui_ctx);
        self.handle_level_view(egui_ctx);
        self.draw_windows(egui_ctx);
    }
}
