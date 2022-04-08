use hv_cell::AtomicRefCell;
use hv_lua::UserData;
use macroquad::audio::play_sound_once;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};
use tealr::mlu::TealData;
use tealr::{TypeBody, TypeName};

use core::Transform;
use std::sync::Arc;

use crate::player::{
    Player, PlayerAttributes, PlayerController, PlayerEventQueue, JUMP_SOUND_ID, LAND_SOUND_ID,
    RESPAWN_DELAY,
};
use crate::{CollisionWorld, Item, Map, PhysicsBody, PlayerEvent, Resources};

const SLIDE_STOP_THRESHOLD: f32 = 2.0;
const JUMP_FRAME_COUNT: u16 = 8;
const PLATFORM_JUMP_FORCE_MULTIPLIER: f32 = 0.2;

#[derive(Debug, Copy, Clone, Eq, PartialEq, TypeName)]
pub enum PlayerState {
    None,
    Jumping,
    Floating,
    Crouching,
    Sliding,
    Incapacitated,
    Dead,
}

impl UserData for PlayerState {}
impl TealData for PlayerState {}
impl TypeBody for PlayerState {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::None
    }
}

pub fn update_player_states(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let query = world.query_mut::<(
        &mut Transform,
        &mut Player,
        &PlayerController,
        &PlayerAttributes,
        &mut PhysicsBody,
    )>();
    for (_, (transform, player, controller, attributes, body)) in query {
        // Timers
        let dt = get_frame_time();

        player.attack_timer -= dt;
        if player.attack_timer <= 0.0 {
            player.attack_timer = 0.0;
        }

        player.is_attacking = player.attack_timer > 0.0;

        player.pickup_grace_timer += dt;

        if player.state == PlayerState::Crouching && !controller.should_crouch {
            player.state = PlayerState::None;
        }

        if player.state == PlayerState::Dead {
            player.respawn_timer += dt;

            player.passive_effects.clear();

            if player.respawn_timer >= RESPAWN_DELAY {
                player.state = PlayerState::None;
                player.respawn_timer = 0.0;

                let map = storage::get::<Map>();
                transform.position = map.get_random_spawn_point();
            }
        } else if player.state == PlayerState::Incapacitated {
            player.incapacitation_timer += dt;

            if player.incapacitation_timer >= attributes.incapacitation_duration {
                player.state = PlayerState::None;
                player.incapacitation_timer = 0.0;
            }
        }

        if player.state == PlayerState::Sliding && body.velocity.x.abs() <= SLIDE_STOP_THRESHOLD {
            body.velocity.x = 0.0;
            player.state = PlayerState::None;
        }

        // Integration
        if player.is_attacking
            || matches!(
                player.state,
                PlayerState::Dead | PlayerState::Incapacitated | PlayerState::Sliding
            )
        {
            body.has_friction = true;

            player.jump_frame_counter = 0;
            body.has_mass = true;
        } else {
            body.has_friction = false;

            if controller.move_direction.x < 0.0 {
                player.is_facing_left = true;
            } else if controller.move_direction.x > 0.0 {
                player.is_facing_left = false;
            }

            if controller.should_slide {
                let velocity = attributes.move_speed * attributes.slide_speed_factor;

                if player.is_facing_left {
                    body.velocity.x = -velocity;
                } else {
                    body.velocity.x = velocity;
                }

                player.state = PlayerState::Sliding;
            } else {
                if controller.move_direction.x < 0.0 {
                    body.velocity.x = -attributes.move_speed;
                } else if controller.move_direction.x > 0.0 {
                    body.velocity.x = attributes.move_speed;
                } else {
                    body.velocity.x = 0.0;
                }

                if controller.should_crouch {
                    if body.is_on_ground {
                        body.velocity.x = 0.0;
                        player.state = PlayerState::Crouching;
                    } else {
                        let mut collision_world = storage::get_mut::<CollisionWorld>();
                        collision_world.descent(body.actor);
                    }
                }

                if body.is_on_ground && controller.should_jump {
                    let jump_force = if controller.should_crouch && body.is_on_platform {
                        attributes.jump_force * PLATFORM_JUMP_FORCE_MULTIPLIER
                    } else {
                        attributes.jump_force
                    };

                    body.velocity.y = -jump_force;

                    player.state = PlayerState::Jumping;

                    let resources = storage::get::<Resources>();
                    let sound = resources.sounds[JUMP_SOUND_ID];

                    play_sound_once(sound);
                } else if player.state == PlayerState::Jumping {
                    player.jump_frame_counter += 1;

                    if controller.should_float && player.jump_frame_counter <= JUMP_FRAME_COUNT {
                        body.has_mass = false;
                    } else {
                        if matches!(player.state, PlayerState::Jumping | PlayerState::Floating) {
                            player.state = PlayerState::None;
                        }

                        player.jump_frame_counter = 0;
                        body.has_mass = true;
                    }
                }

                if !body.is_on_ground && body.velocity.y > 0.0 {
                    if controller.should_float {
                        body.velocity.y *= attributes.float_gravity_factor;
                        player.state = PlayerState::Floating;
                    }
                } else if player.state == PlayerState::Floating {
                    player.state = PlayerState::None;
                }
            }

            if body.is_on_ground && !body.was_on_ground {
                if matches!(player.state, PlayerState::Jumping | PlayerState::Floating) {
                    player.state = PlayerState::None;
                }

                player.jump_frame_counter = 0;
                body.has_mass = true;

                let resources = storage::get::<Resources>();
                let sound = resources.sounds[LAND_SOUND_ID];

                play_sound_once(sound);
            }
        }
    }
}

pub fn update_player_passive_effects(world: Arc<AtomicRefCell<World>>) {
    let mut function_calls = Vec::new();
    {
        let world = AtomicRefCell::borrow_mut(world.as_ref());
        for (entity, (player, events)) in
            world.query::<(&mut Player, &mut PlayerEventQueue)>().iter()
        {
            let dt = get_frame_time();

            for effect in &mut player.passive_effects {
                effect.duration_timer += dt;
            }

            player
                .passive_effects
                .retain(|effect| !effect.is_depleted());

            events.queue.push(PlayerEvent::Update { dt });

            for event in events.queue.iter() {
                let kind = event.into();

                for effect in &mut player.passive_effects {
                    if effect.activated_on.contains(&kind) {
                        effect.use_cnt += 1;

                        if let Some(item_entity) = effect.item {
                            let mut item = world.get_mut::<Item>(item_entity).unwrap();

                            item.use_cnt += 1;
                        }

                        if let Some(f) = &effect.function {
                            function_calls.push((f.to_owned(), entity, effect.item, event.clone()));
                        }
                    }
                }
            }
        }
    }

    for (f, player_entity, item_entity, event) in function_calls.drain(0..) {
        f.call_get_lua(world.clone(), player_entity, item_entity, event);
    }
}

pub fn on_player_damage(world: &mut World, damage_from_entity: Entity, damage_to_entity: Entity) {
    let mut is_from_left = false;

    if let Ok(owner_transform) = world.get::<Transform>(damage_from_entity) {
        if let Ok(target_transform) = world.get::<Transform>(damage_to_entity) {
            is_from_left = owner_transform.position.x < target_transform.position.x;
        }
    }

    {
        let mut events = world
            .get_mut::<PlayerEventQueue>(damage_from_entity)
            .unwrap();

        events.queue.push(PlayerEvent::GiveDamage {
            damage_to: Some(damage_to_entity),
        });
    }

    {
        let mut events = world.get_mut::<PlayerEventQueue>(damage_to_entity).unwrap();

        events.queue.push(PlayerEvent::ReceiveDamage {
            is_from_left,
            damage_from: Some(damage_from_entity),
        });
    }
}
