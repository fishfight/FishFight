use macroquad::{
    color,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        draw_circle, draw_circle_lines, draw_texture_ex, get_frame_time,
        scene::{self, Handle, HandleUntyped, RefMut},
        vec2, Color, DrawTextureParams, Vec2,
    },
};

use crate::Resources;

use super::{
    cannonball::CANNONBALL_HEIGHT,
    player::{capabilities, PhysicsBody, Weapon},
    Player,
};
use crate::nodes::sproinger::Sproingable;

const INITIAL_CANNONBALLS: i32 = 3;
const MAXIMUM_CANNONBALLS: i32 = 3;

const CANNON_WIDTH: f32 = 50.;
const CANNON_HEIGHT: f32 = 30.;
const CANNON_ANIMATION_BASE: &'static str = "base";

const CANNON_THROWBACK: f32 = 1050.0;
const SHOOTING_GRACE_TIME: f32 = 1.0; // seconds

pub struct Cannon {
    cannon_sprite: AnimatedSprite,

    pub thrown: bool,

    pub amount: i32,
    pub body: PhysicsBody,

    grace_time: f32,
}

impl Cannon {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let cannon_sprite = AnimatedSprite::new(
            CANNON_WIDTH as u32,
            CANNON_HEIGHT as u32,
            &[Animation {
                name: CANNON_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            cannon_sprite,
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
                bouncyness: 0.0,
            },
            thrown: false,
            amount: INITIAL_CANNONBALLS,
            grace_time: 0.,
        }
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..MAXIMUM_CANNONBALLS {
            let x = self.body.pos.x + 15.0 * i as f32;

            if i >= self.amount {
                draw_circle_lines(x, self.body.pos.y - 12.0, 4.0, 2., empty_color);
            } else {
                draw_circle(x, self.body.pos.y - 12.0, 4.0, full_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.body.speed = if self.body.facing {
                vec2(600., -200.)
            } else {
                vec2(-600., -200.)
            };
        } else {
            self.body.angle = 3.5;
        }

        let mut resources = storage::get_mut::<Resources>();

        let cannon_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + cannon_mount_pos,
                40,
                30,
            ));
        } else {
            resources.collision_world.set_actor_position(
                self.body.collider.unwrap(),
                self.body.pos + cannon_mount_pos,
            );
        }
    }

    pub fn shoot(node_h: Handle<Cannon>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node_h);

                if node.amount <= 0 || node.grace_time > 0. {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    node.grace_time -= get_frame_time();

                    return;
                } else {
                    node.grace_time = SHOOTING_GRACE_TIME;
                }

                let mut cannonballs =
                    scene::find_node_by_type::<crate::nodes::Cannonballs>().unwrap();
                let cannonball_pos = vec2(
                    node.body.pos.x,
                    node.body.pos.y - 20. - (CANNONBALL_HEIGHT as f32 / 2.),
                );
                cannonballs.spawn_cannonball(cannonball_pos, node.body.facing, player);

                let player = &mut *scene::get_node(player);
                player.body.speed.x = -CANNON_THROWBACK * player.body.facing_dir();
            }

            wait_seconds(0.08).await;

            {
                let mut node = scene::get_node(node_h);

                node.amount -= 1;

                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            Cannon::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Cannon>()
                .handle();

            Cannon::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            node.body.angle = 0.;
            node.amount = INITIAL_CANNONBALLS;

            node.thrown = false;
        }

        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}

impl scene::Node for Cannon {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(CANNON_WIDTH, CANNON_HEIGHT),
            Self::gun_capabilities(),
        ));

        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(CANNON_WIDTH, CANNON_HEIGHT),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.cannon_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();
        }

        node.grace_time -= get_frame_time();
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mut draw_pos = node.body.pos;

        if !node.thrown {
            draw_pos += if node.body.facing {
                vec2(5., 16.)
            } else {
                vec2(-30., 16.)
            }
        };

        draw_texture_ex(
            resources.cannon,
            draw_pos.x,
            draw_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.cannon_sprite.frame().source_rect),
                dest_size: Some(node.cannon_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );

        if node.thrown == false {
            node.draw_hud();
        }
    }
}
