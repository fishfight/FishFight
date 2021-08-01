use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
        scene::{
            Node,
            RefMut,
        },
        collections::storage,
    },
    color,
    prelude::*,
};

use crate::{
    Resources,
    nodes::Player
};

pub struct Seaweed {
    pub sprite: AnimatedSprite,
    pub pos: Vec2,
}

impl Seaweed {
    pub const TRIGGER_WIDTH: f32 = 6.0;
    pub const TRIGGER_HEIGHT: f32 = 16.0;
    pub const TRIGGER_OFFSET_X: f32 = (48.0 - Self::TRIGGER_WIDTH) / 2.0;

    pub const SPEED_THRESHOLD: f32 = 100.0;
    pub const INCAPACITATE_DURATION: f32 = 3.0;

    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            48,
            51,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 5,
                    fps: 8,
                },
            ],
            false,
        );

        Seaweed{
            sprite,
            pos,
        }
    }
}

impl Node for Seaweed {
    fn fixed_update(node: RefMut<Seaweed>) {
        let hitbox = Rect::new(node.pos.x + Seaweed::TRIGGER_OFFSET_X, node.pos.y, Seaweed::TRIGGER_WIDTH, Seaweed::TRIGGER_HEIGHT);
        for mut player in scene::find_nodes_by_type::<Player>() {
            if hitbox.overlaps(&player.get_hitbox()) {
                if player.body.on_ground && (player.body.speed.x >= Seaweed::SPEED_THRESHOLD || player.body.speed.x <= -Seaweed::SPEED_THRESHOLD) {
                    player.incapacitate(Seaweed::INCAPACITATE_DURATION, true);
                }
            }
        }
    }

    fn draw(mut node: RefMut<Seaweed>) {
        node.sprite.update();

        let resources = storage::get_mut::<Resources>();

        draw_texture_ex(
            resources.seaweed,
            node.pos.x,
            node.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
