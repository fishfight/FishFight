use macroquad::{
    //audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::{
        player::{capabilities, PhysicsBody, Weapon},
        sproinger::Sproingable,
        ArmedMine, Player,
    },
    Resources,
};

pub struct Mines {
    mines_sprite: AnimatedSprite,

    pub thrown: bool,

    pub amount: i32,
    pub body: PhysicsBody,
}

impl Mines {
    pub const COLLIDER_WIDTH: f32 = 32.0;
    pub const COLLIDER_HEIGHT: f32 = 16.0;
    pub const FIRE_INTERVAL: f32 = 0.5;
    pub const MAXIMUM_AMOUNT: i32 = 3;

    pub fn new(facing: bool, pos: Vec2) -> Self {
        let mines_sprite = AnimatedSprite::new(
            26,
            40,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Mines {
            mines_sprite,
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
            amount: Self::MAXIMUM_AMOUNT,
        }
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAXIMUM_AMOUNT {
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

        let mines_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + mines_mount_pos,
                15,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + mines_mount_pos);
        }
    }

    pub fn shoot(node: Handle<Mines>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node);
                if node.amount <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }

                ArmedMine::spawn(node.body.pos, node.body.facing);
                node.amount -= 1;
            }

            wait_seconds(Mines::FIRE_INTERVAL).await;

            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            Mines::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Mines>()
                .handle();

            Mines::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();

            node.body.angle = 0.;
            node.amount = Mines::MAXIMUM_AMOUNT;
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

impl scene::Node for Mines {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(32.0, 28.0),
            Self::gun_capabilities(),
        ));

        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(32.0, 28.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.mines_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if !node.body.on_ground {
                let hitbox = Rect::new(
                    node.body.pos.x,
                    node.body.pos.y,
                    Mines::COLLIDER_WIDTH,
                    Mines::COLLIDER_HEIGHT,
                );
                for mut player in scene::find_nodes_by_type::<Player>() {
                    if hitbox.overlaps(&player.get_hitbox()) {
                        if let Some((weapon, _, _, gun)) = player.weapon.as_mut() {
                            (gun.throw)(*weapon, false);
                            player.weapon = None;
                        }
                    }
                }
            }
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mine_mount_pos = if !node.thrown {
            if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-5., 16.)
            }
        } else if node.body.facing {
            vec2(-25., -10.)
        } else {
            vec2(5., -10.)
        };

        draw_texture_ex(
            resources.mines,
            node.body.pos.x + mine_mount_pos.x,
            node.body.pos.y + mine_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.mines_sprite.frame().source_rect),
                dest_size: Some(node.mines_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );

        if !node.thrown {
            node.draw_hud();
        }
    }
}
