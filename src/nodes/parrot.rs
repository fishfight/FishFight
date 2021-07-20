use macroquad::{
    color,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        draw_circle, draw_circle_lines, draw_texture_ex, get_frame_time,
        scene::{self, Handle, HandleUntyped, RefMut},
        vec2, Color, DrawTextureParams, Rect, Vec2,
    },
};

use crate::Resources;

use super::{
    player::{capabilities, PhysicsBody, Weapon, PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH},
    Player,
};

const INITIAL_FLYING_PARROTS: i32 = 3;
const MAXIMUM_FLYING_PARROTS: i32 = 3;

const PARROT_WIDTH: f32 = 19.;
const PARROT_HEIGHT: f32 = 24.;
const PARROT_ANIMATION_BASE: &'static str = "base";
const PARROT_MOUNT_X_REL: f32 = -12.;
const PARROT_MOUNT_Y: f32 = -10.;

const SHOOTING_GRACE_TIME: f32 = 1.0; // seconds

pub struct Parrot {
    parrot_sprite: AnimatedSprite,

    pub thrown: bool,

    pub amount: i32,
    pub body: PhysicsBody,

    origin_pos: Vec2,
    deadly_dangerous: bool,

    grace_time: f32,
}

impl Parrot {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let parrot_sprite = AnimatedSprite::new(
            PARROT_WIDTH as u32,
            PARROT_HEIGHT as u32,
            &[Animation {
                name: PARROT_ANIMATION_BASE.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Self {
            parrot_sprite,
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
            },
            thrown: false,
            amount: INITIAL_FLYING_PARROTS,
            origin_pos: pos,
            deadly_dangerous: false,
            grace_time: 0.,
        }
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..MAXIMUM_FLYING_PARROTS {
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

        let parrot_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + parrot_mount_pos,
                40,
                30,
            ));
        } else {
            resources.collision_world.set_actor_position(
                self.body.collider.unwrap(),
                self.body.pos + parrot_mount_pos,
            );
        }
        self.origin_pos = self.body.pos + parrot_mount_pos / 2.;
    }

    pub fn shoot(node_h: Handle<Parrot>, player: Handle<Player>) -> Coroutine {
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

                let mut flying_parrots =
                    scene::find_node_by_type::<crate::nodes::FlyingParrots>().unwrap();
                flying_parrots.spawn_flying_parrot(&node.body);
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
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Parrot>();

            Parrot::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Parrot>()
                .handle();

            Parrot::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Parrot>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Parrot>();

            node.body.angle = 0.;
            node.amount = INITIAL_FLYING_PARROTS;

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

impl scene::Node for Parrot {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.parrot_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground {
                node.deadly_dangerous = false;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let parrot_hit_box = Rect::new(
                    node.body.pos.x,
                    node.body.pos.y,
                    PARROT_WIDTH,
                    PARROT_HEIGHT,
                );

                for mut other in others {
                    if Rect::new(
                        other.body.pos.x,
                        other.body.pos.y,
                        PLAYER_HITBOX_WIDTH,
                        PLAYER_HITBOX_HEIGHT,
                    )
                    .overlaps(&parrot_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }

        node.grace_time -= get_frame_time();
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let parrot_mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(PARROT_MOUNT_X_REL, PARROT_MOUNT_Y)
            } else {
                vec2(-PARROT_MOUNT_X_REL, PARROT_MOUNT_Y)
            }
        } else {
            if node.body.facing {
                vec2(-PARROT_WIDTH, 0.)
            } else {
                vec2(PARROT_WIDTH, 0.)
            }
        };

        draw_texture_ex(
            resources.parrot,
            node.body.pos.x + parrot_mount_pos.x,
            node.body.pos.y + parrot_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.parrot_sprite.frame().source_rect),
                dest_size: Some(node.parrot_sprite.frame().dest_size),
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
