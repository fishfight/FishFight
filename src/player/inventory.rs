use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, ToLua};
use macroquad::prelude::*;

use hecs::{Entity, With, Without, World};
use tealr::{TypeBody, TypeName};

use core::lua::get_table;
use core::lua::wrapped_types::Vec2Lua;
use core::Transform;
use std::borrow::Cow;
use std::sync::Arc;

use crate::items::{
    fire_weapon, ItemDepleteBehavior, ItemDropBehavior, Weapon, EFFECT_ANIMATED_SPRITE_ID,
    GROUND_ANIMATION_ID, ITEMS_DRAW_ORDER, SPRITE_ANIMATED_SPRITE_ID,
};
use crate::particles::ParticleEmitter;
use crate::player::{Player, PlayerController, PlayerState, IDLE_ANIMATION_ID, PICKUP_GRACE_TIME};
use crate::{Drawable, Item, Owner, PassiveEffectInstance, PhysicsBody};

const THROW_FORCE: f32 = 5.0;

#[derive(Clone, TypeName, Default)]
pub struct PlayerInventory {
    pub weapon_mount: Vec2,
    pub weapon_mount_offset: Vec2,
    pub item_mount: Vec2,
    pub item_mount_offset: Vec2,
    pub hat_mount: Vec2,
    pub hat_mount_offset: Vec2,
    pub weapon: Option<Entity>,
    pub items: Vec<Entity>,
    pub hat: Option<Entity>,
}

impl<'lua> FromLua<'lua> for PlayerInventory {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            weapon_mount: table.get::<_, Vec2Lua>("weapon_mount")?.into(),
            weapon_mount_offset: table.get::<_, Vec2Lua>("weapon_mount_offset")?.into(),
            item_mount: table.get::<_, Vec2Lua>("item_mount")?.into(),
            item_mount_offset: table.get::<_, Vec2Lua>("item_mount_offset")?.into(),
            hat_mount: table.get::<_, Vec2Lua>("hat_mount")?.into(),
            hat_mount_offset: table.get::<_, Vec2Lua>("hat_mount_offset")?.into(),
            weapon: table.get("weapon")?,
            items: table.get("items")?,
            hat: table.get("hat")?,
        })
    }
}
impl<'lua> ToLua<'lua> for PlayerInventory {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("weapon_mount", Vec2Lua::from(self.weapon_mount))?;
        table.set(
            "weapon_mount_offset",
            Vec2Lua::from(self.weapon_mount_offset),
        )?;
        table.set("item_mount", Vec2Lua::from(self.item_mount))?;
        table.set("item_mount_offset", Vec2Lua::from(self.item_mount_offset))?;
        table.set("hat_mount", Vec2Lua::from(self.hat_mount))?;
        table.set("hat_mount_offset", Vec2Lua::from(self.hat_mount_offset))?;
        table.set("weapon", self.weapon)?;
        table.set("items", self.items)?;
        table.set("hat", self.hat)?;
        lua.pack(table)
    }
}

impl TypeBody for PlayerInventory {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("weapon_mount").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("weapon_mount_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("item_mount").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("item_mount_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("hat_mount").into(), Vec2Lua::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("hat_mount_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("weapon").into(),
            Option::<Entity>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("items").into(),
            Vec::<Entity>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("hat").into(),
            Option::<Entity>::get_type_parts(),
        ));
    }
}

impl PlayerInventory {
    pub fn new(weapon_mount: Vec2, item_mount: Vec2, hat_mount: Vec2) -> Self {
        PlayerInventory {
            weapon_mount,
            weapon_mount_offset: Vec2::ZERO,
            item_mount,
            item_mount_offset: Vec2::ZERO,
            hat_mount,
            hat_mount_offset: Vec2::ZERO,
            weapon: None,
            items: Vec::new(),
            hat: None,
        }
    }

    pub fn get_weapon_mount(&self, is_facing_left: bool, is_upside_down: bool) -> Vec2 {
        flip_offset(
            self.weapon_mount + self.weapon_mount_offset,
            None,
            is_facing_left,
            is_upside_down,
        )
    }
}

pub fn update_player_inventory(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let mut item_colliders = world
        .query::<With<Item, Without<Owner, (&Transform, &PhysicsBody)>>>()
        .iter()
        .map(|(e, (transform, body))| (e, body.as_rect(transform.position)))
        .collect::<Vec<_>>();

    let mut weapon_colliders = world
        .query::<With<Weapon, Without<Owner, (&Transform, &PhysicsBody)>>>()
        .iter()
        .map(|(e, (transform, body))| (e, body.as_rect(transform.position)))
        .collect::<Vec<_>>();

    let mut picked_up = Vec::new();

    let mut to_drop = Vec::new();
    let mut to_fire = Vec::new();
    let mut to_destroy = Vec::new();

    for (entity, (transform, player, controller, inventory, body)) in world
        .query::<(
            &mut Transform,
            &mut Player,
            &PlayerController,
            &mut PlayerInventory,
            &mut PhysicsBody,
        )>()
        .iter()
    {
        if player.state == PlayerState::Dead {
            for item_entity in inventory.items.drain(0..) {
                to_drop.push(item_entity);
            }

            if let Some(weapon_entity) = inventory.weapon.take() {
                to_drop.push(weapon_entity);
            }
        } else {
            let player_rect = body.as_rect(transform.position);

            let mut i = 0;
            while i < item_colliders.len() {
                let &(item_entity, rect) = item_colliders.get(i).unwrap();

                if player_rect.overlaps(&rect) {
                    let item = world.get::<Item>(item_entity).unwrap();

                    if item.is_hat {
                        if player.pickup_grace_timer < PICKUP_GRACE_TIME {
                            i += 1;

                            continue;
                        }

                        if let Some(hat_entity) = inventory.hat.take() {
                            to_drop.push(hat_entity);
                        }

                        inventory.hat = Some(item_entity);

                        player.pickup_grace_timer = 0.0;
                    } else {
                        inventory.items.push(item_entity);
                    }

                    picked_up.push((entity, item_entity));
                    item_colliders.remove(i);

                    let mut body = world.get_mut::<PhysicsBody>(item_entity).unwrap();
                    body.is_deactivated = true;

                    let mut drawable = world.get_mut::<Drawable>(item_entity).unwrap();
                    let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();
                    sprite_set.set_all(IDLE_ANIMATION_ID, true);

                    continue;
                }

                i += 1;
            }

            if controller.should_pickup {
                if let Some(weapon_entity) = inventory.weapon.take() {
                    to_drop.push(weapon_entity);

                    let velocity = if player.is_facing_left {
                        vec2(-THROW_FORCE, 0.0)
                    } else {
                        vec2(THROW_FORCE, 0.0)
                    };

                    let mut body = world.get_mut::<PhysicsBody>(weapon_entity).unwrap();

                    body.velocity = velocity;
                } else if player.pickup_grace_timer >= PICKUP_GRACE_TIME {
                    for (i, &(weapon_entity, rect)) in weapon_colliders.iter().enumerate() {
                        if player_rect.overlaps(&rect) {
                            picked_up.push((entity, weapon_entity));
                            weapon_colliders.remove(i);

                            inventory.weapon = Some(weapon_entity);
                            player.pickup_grace_timer = 0.0;

                            let mut body = world.get_mut::<PhysicsBody>(weapon_entity).unwrap();
                            body.is_deactivated = true;

                            let mut drawable = world.get_mut::<Drawable>(weapon_entity).unwrap();
                            let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

                            sprite_set.set_all(IDLE_ANIMATION_ID, true);

                            break;
                        }
                    }
                }
            }

            if let Some(weapon_entity) = inventory.weapon {
                let mut weapon = world.get_mut::<Weapon>(weapon_entity).unwrap();

                weapon.cooldown_timer += get_frame_time();

                let mut weapon_transform = world.get_mut::<Transform>(weapon_entity).unwrap();

                let weapon_mount = transform.position
                    + inventory.get_weapon_mount(player.is_facing_left, player.is_upside_down);

                weapon_transform.position = weapon_mount;

                let mut drawable = world.get_mut::<Drawable>(weapon_entity).unwrap();
                let frame_size = {
                    let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

                    sprite_set.flip_all_x(player.is_facing_left);
                    sprite_set.flip_all_y(player.is_upside_down);

                    sprite_set
                        .map
                        .get(SPRITE_ANIMATED_SPRITE_ID)
                        .map(|sprite| sprite.size())
                        .unwrap()
                };

                let mount_offset = flip_offset(
                    weapon.mount_offset,
                    frame_size,
                    player.is_facing_left,
                    player.is_upside_down,
                );

                weapon_transform.position += mount_offset;

                if let Ok(mut particle_emitters) =
                    world.get_mut::<Vec<ParticleEmitter>>(weapon_entity)
                {
                    let mut offset = weapon.effect_offset;

                    if player.is_facing_left {
                        offset.x = frame_size.x - offset.x;
                    }

                    if player.is_upside_down {
                        offset.y = frame_size.y - offset.y;
                    }

                    for emitter in particle_emitters.iter_mut() {
                        emitter.offset = offset;
                    }
                }

                let is_depleted = weapon
                    .uses
                    .map(|uses| weapon.use_cnt >= uses)
                    .unwrap_or_default();

                if is_depleted {
                    match weapon.deplete_behavior {
                        ItemDepleteBehavior::Destroy => {
                            to_destroy.push(weapon_entity);
                            inventory.weapon = None;
                        }
                        ItemDepleteBehavior::Drop => {
                            to_drop.push(weapon_entity);
                            inventory.weapon = None;
                        }
                        _ => {}
                    }
                } else if controller.should_attack {
                    to_fire.push((weapon_entity, entity));
                }
            }

            let mut handle_item = |item_entity| -> bool {
                let mut res = false;

                let mut item = world.get_mut::<Item>(item_entity).unwrap();

                item.duration_timer += get_frame_time();

                let mut is_depleted = false;

                if let Some(uses) = item.uses {
                    is_depleted = item.use_cnt >= uses;
                }

                if let Some(duration) = item.duration {
                    is_depleted = is_depleted || item.duration_timer >= duration;
                }

                if is_depleted {
                    res = true;

                    player.passive_effects.retain(|effect| {
                        if let Some(effect_item_entity) = effect.item {
                            effect_item_entity != item_entity
                        } else {
                            true
                        }
                    });
                } else {
                    let position = transform.position;

                    match world.get_mut::<Drawable>(item_entity) {
                        Ok(mut drawable) => {
                            let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

                            sprite_set.flip_all_x(player.is_facing_left);
                            sprite_set.flip_all_y(player.is_upside_down);

                            let mount_offset = if item.is_hat {
                                let size = sprite_set.size();

                                vec2(-size.x / 2.0, size.y)
                                    + inventory.hat_mount
                                    + inventory.hat_mount_offset
                            } else {
                                inventory.item_mount + inventory.item_mount_offset
                            };

                            let offset = flip_offset(
                                item.mount_offset + mount_offset,
                                sprite_set.size(),
                                player.is_facing_left,
                                player.is_upside_down,
                            );

                            let mut item_transform =
                                world.get_mut::<Transform>(item_entity).unwrap();
                            item_transform.position = position + offset;
                        }
                        Err(err) => {
                            #[cfg(debug_assertions)]
                            println!("WARNING: {}", err);
                        }
                    }
                }

                res
            };

            if let Some(hat_entity) = inventory.hat.take() {
                if handle_item(hat_entity) {
                    to_drop.push(hat_entity);
                } else {
                    inventory.hat = Some(hat_entity);
                }
            }

            let mut i = 0;
            while i < inventory.items.len() {
                let item_entity = *inventory.items.get(i).unwrap();

                if handle_item(item_entity) {
                    inventory.items.remove(i);
                    to_drop.push(item_entity);
                } else {
                    i += 1;
                }
            }
        }
    }

    for (player_entity, item_entity) in picked_up {
        world.insert_one(item_entity, Owner(player_entity)).unwrap();

        let player_draw_order = world
            .get::<Drawable>(player_entity)
            .map(|drawable| drawable.draw_order)
            .unwrap();

        let mut drawable = world.get_mut::<Drawable>(item_entity).unwrap();
        drawable.draw_order = player_draw_order + 1;

        if let Ok(item) = world.get::<Item>(item_entity) {
            let mut player = world.get_mut::<Player>(player_entity).unwrap();

            for meta in item.effects.clone().into_iter() {
                let effect_instance = PassiveEffectInstance::new(Some(item_entity), meta);
                player.passive_effects.push(effect_instance);
            }
        }
    }

    for entity in to_drop {
        world.remove_one::<Owner>(entity).unwrap();

        let mut should_destroy = false;

        if let Ok(mut weapon) = world.get_mut::<Weapon>(entity) {
            match weapon.drop_behavior {
                ItemDropBehavior::ClearState => {
                    weapon.use_cnt = 0;
                    weapon.cooldown_timer = weapon.cooldown;
                }
                ItemDropBehavior::Destroy => {
                    should_destroy = true;
                }
                _ => {}
            }
        } else if let Ok(mut item) = world.get_mut::<Item>(entity) {
            match item.drop_behavior {
                ItemDropBehavior::ClearState => {
                    item.use_cnt = 0;
                    item.duration_timer = 0.0;
                }
                ItemDropBehavior::PersistState => {
                    if let Some(uses) = item.uses {
                        should_destroy = item.use_cnt >= uses;
                    }
                }
                ItemDropBehavior::Destroy => {
                    should_destroy = true;
                }
            }
        }

        if should_destroy {
            if let Err(err) = world.despawn(entity) {
                #[cfg(debug_assertions)]
                println!("WARNING: {}", err);
            }
        } else {
            let mut drawable = world.get_mut::<Drawable>(entity).unwrap();
            drawable.draw_order = ITEMS_DRAW_ORDER;

            let mut body = world.get_mut::<PhysicsBody>(entity).unwrap();

            body.is_deactivated = false;

            let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

            sprite_set.restart_all();

            let sprite = sprite_set.map.get_mut(SPRITE_ANIMATED_SPRITE_ID).unwrap();

            if sprite
                .animations
                .iter()
                .any(|a| a.id == *GROUND_ANIMATION_ID)
            {
                sprite.set_animation(GROUND_ANIMATION_ID, true);
            } else {
                sprite.set_animation(IDLE_ANIMATION_ID, true);
            }

            if let Some(sprite) = sprite_set.map.get_mut(EFFECT_ANIMATED_SPRITE_ID) {
                sprite.is_deactivated = true;
            }
        }
    }

    for (entity, owner) in to_fire.drain(0..) {
        if let Err(err) = fire_weapon(&mut world, entity, owner) {
            #[cfg(debug_assertions)]
            println!("WARNING: {}", err);
        }
    }

    for entity in to_destroy {
        if let Err(err) = world.despawn(entity) {
            #[cfg(debug_assertions)]
            println!("WARNING: {}", err);
        }
    }
}

const HUD_OFFSET_Y: f32 = 16.0;

const HUD_CONDENSED_USE_COUNT_THRESHOLD: u32 = 12;

const HUD_USE_COUNT_COLOR_FULL: Color = Color {
    r: 0.8,
    g: 0.9,
    b: 1.0,
    a: 1.0,
};

const HUD_USE_COUNT_COLOR_EMPTY: Color = Color {
    r: 0.8,
    g: 0.9,
    b: 1.0,
    a: 0.8,
};

pub fn draw_weapons_hud(world: Arc<AtomicRefCell<World>>) {
    let world = AtomicRefCell::borrow(world.as_ref());
    for (_, (transform, inventory)) in world.query::<(&Transform, &PlayerInventory)>().iter() {
        if let Some(weapon_entity) = inventory.weapon {
            let weapon = world.get::<Weapon>(weapon_entity).unwrap();
            if let Some(uses) = weapon.uses {
                let is_destroyed_on_depletion =
                    weapon.deplete_behavior == ItemDepleteBehavior::Destroy;

                if !is_destroyed_on_depletion || uses > 1 {
                    let remaining = uses - weapon.use_cnt;

                    let mut position = transform.position;
                    position.y -= HUD_OFFSET_Y;

                    if uses >= HUD_CONDENSED_USE_COUNT_THRESHOLD {
                        let x = position.x - ((4.0 * uses as f32) / 2.0);

                        for i in 0..uses {
                            draw_rectangle(
                                x + 4.0 * i as f32,
                                position.y - 12.0,
                                2.0,
                                12.0,
                                if i >= remaining {
                                    HUD_USE_COUNT_COLOR_EMPTY
                                } else {
                                    HUD_USE_COUNT_COLOR_FULL
                                },
                            )
                        }
                    } else {
                        let x = position.x - (uses as f32 * 14.0) / 2.0;

                        for i in 0..uses {
                            let x = x + 14.0 * i as f32;

                            if i >= remaining {
                                draw_circle_lines(
                                    x,
                                    position.y - 4.0,
                                    4.0,
                                    2.0,
                                    HUD_USE_COUNT_COLOR_EMPTY,
                                );
                            } else {
                                draw_circle(x, position.y - 4.0, 4.0, HUD_USE_COUNT_COLOR_FULL);
                            };
                        }
                    }
                }
            }
        }
    }
}

pub fn flip_offset<S: Into<Option<Vec2>>>(
    offset: Vec2,
    size: S,
    is_facing_left: bool,
    is_upside_down: bool,
) -> Vec2 {
    let mut corrected = Vec2::ZERO;

    let size = size.into().unwrap_or_default();

    if is_facing_left {
        corrected.x -= offset.x + size.x;
    } else {
        corrected.x = offset.x;
    }

    if is_upside_down {
        corrected.y -= offset.y + size.y;
    } else {
        corrected.y = offset.y;
    }

    corrected
}
