use hv_cell::AtomicRefCell;
use macroquad::audio::play_sound_once;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use hecs::{Entity, World};

use core::Result;
use core::Transform;

use crate::{Animation, Drawable, PhysicsBody, QueuedAnimationAction, Resources};

const SPROINGER_DRAW_ORDER: u32 = 2;

const TEXTURE_ID: &str = "sproinger";

const IDLE_ANIMATION_ID: &str = "idle";
const EXPAND_ANIMATION_ID: &str = "expand";
const CONTRACT_ANIMATION_ID: &str = "contract";

const SOUND_EFFECT_ID: &str = "jump";

const COOLDOWN: f32 = 0.75;

const TRIGGER_WIDTH: f32 = 32.0;
const TRIGGER_HEIGHT: f32 = 8.0;

const FORCE: f32 = 25.0;

#[derive(Default)]
pub struct Sproinger {
    pub cooldown_timer: f32,
}

impl Sproinger {
    pub fn new() -> Self {
        Sproinger {
            cooldown_timer: COOLDOWN,
        }
    }
}

pub fn spawn_sproinger(world: &mut World, position: Vec2) -> Result<Entity> {
    let animations = &[
        Animation {
            id: IDLE_ANIMATION_ID.to_string(),
            row: 0,
            frames: 1,
            fps: 1,
            tweens: HashMap::new(),
            is_looping: false,
        },
        Animation {
            id: EXPAND_ANIMATION_ID.to_string(),
            row: 1,
            frames: 2,
            fps: 8,
            tweens: HashMap::new(),
            is_looping: false,
        },
        Animation {
            id: CONTRACT_ANIMATION_ID.to_string(),
            row: 2,
            frames: 2,
            fps: 4,
            tweens: HashMap::new(),
            is_looping: false,
        },
    ];

    let entity = world.spawn((
        Sproinger::new(),
        Transform::from(position),
        Drawable::new_animated_sprite(
            SPROINGER_DRAW_ORDER,
            TEXTURE_ID,
            animations,
            Default::default(),
        ),
    ));

    Ok(entity)
}

pub fn fixed_update_sproingers(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let dt = get_frame_time();

    let bodies = world
        .query::<(&Transform, &PhysicsBody)>()
        .iter()
        .filter_map(|(e, (transform, body))| {
            if !body.is_deactivated {
                Some((e, body.as_rect(transform.position)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut to_be_sproinged = Vec::new();

    'sproingers: for (_, (sproinger, transform, drawable)) in
        world.query_mut::<(&mut Sproinger, &Transform, &mut Drawable)>()
    {
        sproinger.cooldown_timer += dt;

        let sound = {
            let resources = storage::get::<Resources>();
            resources.sounds[SOUND_EFFECT_ID]
        };

        if sproinger.cooldown_timer >= COOLDOWN {
            let sprite = drawable.get_animated_sprite_mut().unwrap();
            sprite.set_animation(IDLE_ANIMATION_ID, true);

            let position = transform.position - (sprite.frame_size / 2.0);

            let trigger_rect = Rect::new(position.x, position.y, TRIGGER_WIDTH, TRIGGER_HEIGHT);

            for (e, rect) in &bodies {
                if trigger_rect.overlaps(rect) {
                    to_be_sproinged.push(*e);

                    sproinger.cooldown_timer = 0.0;

                    sprite.set_animation(EXPAND_ANIMATION_ID, true);
                    sprite.queue_action(QueuedAnimationAction::Play(
                        CONTRACT_ANIMATION_ID.to_string(),
                    ));

                    play_sound_once(sound);

                    continue 'sproingers;
                }
            }
        }
    }

    for entity in to_be_sproinged {
        if let Ok(mut body) = world.get_mut::<PhysicsBody>(entity) {
            body.velocity.y = -FORCE;
        }
    }
}
