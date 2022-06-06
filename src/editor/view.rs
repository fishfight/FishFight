use super::util::EguiCompatibleVec;

#[derive(Clone, Copy)]
pub struct LevelView {
    /// The view offset in pixels.
    pub position: macroquad::prelude::Vec2,
    /// The scale the level is viewed with. 1.0 == 1:1, bigger numbers mean bigger tiles.
    pub scale: f32,
}

impl Default for LevelView {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: 1.,
        }
    }
}

impl LevelView {
    // TODO: Factor in level view scale
    pub fn screen_to_world_pos(&self, p: egui::Pos2) -> egui::Pos2 {
        (p.to_vec2() / self.scale + self.position.into_egui()).to_pos2()
    }

    pub fn world_to_screen_pos(&self, p: egui::Pos2) -> egui::Pos2 {
        ((p.to_vec2() - self.position.into_egui()) * self.scale).to_pos2()
    }
}

#[derive(Clone)]
pub struct UiLevelView {
    pub view: LevelView,
    pub response: egui::Response,
    painter: egui::Painter,
}

impl UiLevelView {
    pub fn new(view: LevelView, response: egui::Response, painter: egui::Painter) -> Self {
        Self {
            view,
            response,
            painter,
        }
    }

    /// Get a reference to the ui level view's painter.
    pub fn painter(&self) -> &egui::Painter {
        &self.painter
    }

    pub fn ctx(&self) -> &egui::Context {
        self.painter.ctx()
    }

    /// Returns the top left pixel of the level rect in screen coordinates.
    pub fn level_top_left(&self) -> egui::Pos2 {
        self.response.rect.min
    }

    // TODO: Factor in level view scale
    pub fn screen_to_world_pos(&self, p: egui::Pos2) -> egui::Pos2 {
        self.view
            .screen_to_world_pos(p - self.level_top_left().to_vec2())
    }

    pub fn world_to_screen_pos(&self, p: egui::Pos2) -> egui::Pos2 {
        self.view.world_to_screen_pos(p) + self.level_top_left().to_vec2()
    }
}
