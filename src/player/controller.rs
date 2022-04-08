use hecs::World;

use hv_cell::AtomicRefCell;
use macroquad::prelude::*;

use core::network::PlayerId;

use core::input::{collect_local_input, GameInputScheme, PlayerInput};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PlayerControllerKind {
    LocalInput(GameInputScheme),
    Network(PlayerId),
}

impl PlayerControllerKind {
    pub fn is_local(&self) -> bool {
        matches!(self, PlayerControllerKind::LocalInput(..))
    }
}

pub struct PlayerController {
    pub kind: PlayerControllerKind,

    /// No vertical movement is possible now but you never know what the future holds :)
    pub move_direction: Vec2,

    pub should_crouch: bool,
    pub should_jump: bool,
    pub should_float: bool,
    pub should_pickup: bool,
    pub should_attack: bool,
    pub should_slide: bool,
}

impl From<PlayerControllerKind> for PlayerController {
    fn from(kind: PlayerControllerKind) -> Self {
        PlayerController {
            kind,
            move_direction: Vec2::ZERO,
            should_crouch: false,
            should_jump: false,
            should_float: false,
            should_pickup: false,
            should_attack: false,
            should_slide: false,
        }
    }
}

impl PlayerController {
    pub fn clear(&mut self) {
        self.move_direction = Vec2::ZERO;
        self.should_crouch = false;
        self.should_jump = false;
        self.should_float = false;
        self.should_pickup = false;
        self.should_attack = false;
        self.should_slide = false;
    }

    pub fn apply_input(&mut self, input: PlayerInput) {
        self.clear();

        if input.left {
            self.move_direction.x -= 1.0;
        }

        if input.right {
            self.move_direction.x += 1.0;
        }

        self.should_crouch = input.crouch;
        self.should_jump = input.jump;
        self.should_float = input.float;
        self.should_pickup = input.pickup;
        self.should_attack = input.fire;
        self.should_slide = input.slide;
    }
}

pub fn update_player_controllers(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, controller) in world.query_mut::<&mut PlayerController>() {
        let input = match &controller.kind {
            PlayerControllerKind::LocalInput(input_scheme) => collect_local_input(*input_scheme),
            PlayerControllerKind::Network(_player_id) => PlayerInput::default(),
        };

        controller.apply_input(input);
    }
}
