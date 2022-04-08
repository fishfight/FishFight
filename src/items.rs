//! Things available to spawn from the level editor
//! Proto-mods, eventually some of the items will move to some sort of a wasm runtime

use hecs::{Entity, World};
use hv_lua::{FromLua, ToLua};
use macroquad::audio::{play_sound_once, Sound};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};
use tealr::{TypeBody, TypeName};

use crate::{
    ActiveEffectMetadata, AnimatedSprite, AnimatedSpriteMetadata, CollisionWorld, Drawable,
    PassiveEffectMetadata, PhysicsBody, QueuedAnimationAction, Resources,
};

use core::lua::get_table;
use core::lua::wrapped_types::{SoundLua, Vec2Lua};
use core::{Result, Transform};
use std::borrow::Cow;

use crate::effects::active::spawn_active_effect;
use crate::particles::{ParticleEmitter, ParticleEmitterMetadata};
use crate::physics::PhysicsBodyParams;
use crate::player::{Player, PlayerInventory, IDLE_ANIMATION_ID};

pub const ITEMS_DRAW_ORDER: u32 = 1;

pub const SPRITE_ANIMATED_SPRITE_ID: &str = "sprite";
pub const EFFECT_ANIMATED_SPRITE_ID: &str = "effect";

pub const GROUND_ANIMATION_ID: &str = "ground";
pub const ATTACK_ANIMATION_ID: &str = "attack";

/// This dictates what happens to an item when it is dropped, either manually or on death.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TypeName)]
#[serde(rename_all = "snake_case")]
pub enum ItemDropBehavior {
    /// Clear all state and restore default parameters
    ClearState,
    /// Keep all state between pickups
    PersistState,
    /// Destroy the item on drop
    Destroy,
}

impl<'lua> FromLua<'lua> for ItemDropBehavior {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        hv_lua::LuaSerdeExt::from_value(lua, lua_value)
    }
}

impl<'lua> ToLua<'lua> for ItemDropBehavior {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        hv_lua::LuaSerdeExt::to_value(lua, &self)
    }
}
impl TypeBody for ItemDropBehavior {
    fn get_type_body(_: &mut tealr::TypeGenerator) {}
}

impl Default for ItemDropBehavior {
    fn default() -> Self {
        ItemDropBehavior::ClearState
    }
}

/// This dictates what happens to an item when it is depleted, either by exceeding its duration,
/// in the case of `Equipment`, or by depleting `uses`, if specified, in the case of a `Weapon`
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TypeName)]
#[serde(rename_all = "snake_case")]
pub enum ItemDepleteBehavior {
    /// Keep the item on depletion (do nothing)
    Keep,
    /// Drop the item on depletion
    Drop,
    /// Destroy the item on depletion
    Destroy,
}

impl<'lua> FromLua<'lua> for ItemDepleteBehavior {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        hv_lua::LuaSerdeExt::from_value(lua, lua_value)
    }
}

impl<'lua> ToLua<'lua> for ItemDepleteBehavior {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        hv_lua::LuaSerdeExt::to_value(lua, &self)
    }
}
impl TypeBody for ItemDepleteBehavior {
    fn get_type_body(_: &mut tealr::TypeGenerator) {}
}

impl Default for ItemDepleteBehavior {
    fn default() -> Self {
        ItemDepleteBehavior::Keep
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapItemKind {
    Weapon {
        #[serde(flatten)]
        meta: WeaponMetadata,
    },
    Item {
        #[serde(flatten)]
        meta: ItemMetadata,
    },
}

pub struct ItemParams {
    pub name: String,
    pub effects: Vec<PassiveEffectMetadata>,
    pub uses: Option<u32>,
    pub duration: Option<f32>,
    pub mount_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
    pub is_hat: bool,
}

#[derive(Clone, TypeName)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub effects: Vec<PassiveEffectMetadata>,
    pub uses: Option<u32>,
    pub duration: Option<f32>,
    pub mount_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
    pub is_hat: bool,
    pub duration_timer: f32,
    pub use_cnt: u32,
}

impl<'lua> FromLua<'lua> for Item {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            id: table.get("id")?,
            name: table.get("name")?,
            effects: table.get("effects")?,
            uses: table.get("uses")?,
            duration: table.get("duration")?,
            mount_offset: table.get::<_, Vec2Lua>("mount_offset")?.into(),
            drop_behavior: table.get("drop_behavior")?,
            deplete_behavior: table.get("deplete_behavior")?,
            is_hat: table.get("is_hat")?,
            duration_timer: table.get("duration_timer")?,
            use_cnt: table.get("use_cnt")?,
        })
    }
}

impl<'lua> ToLua<'lua> for Item {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("name", self.name)?;
        table.set("effects", self.effects)?;
        table.set("uses", self.uses)?;
        table.set("duration", self.duration)?;
        table.set("mount_offset", Vec2Lua::from(self.mount_offset))?;
        table.set("drop_behavior", self.drop_behavior)?;
        table.set("deplete_behavior", self.deplete_behavior)?;
        table.set("is_hat", self.is_hat)?;
        table.set("duration_timer", self.duration_timer)?;
        table.set("use_cnt", self.use_cnt)?;
        lua.pack(table)
    }
}

impl TypeBody for Item {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("id").into(), String::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("name").into(), String::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("effects").into(),
            Vec::<PassiveEffectMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("uses").into(),
            Option::<u32>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("duration").into(),
            Option::<f32>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("mount_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("drop_behavior").into(),
            ItemDropBehavior::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("deplete_behavior").into(),
            ItemDepleteBehavior::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_hat").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("duration_timer").into(),
            f32::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("use_cnt").into(), u32::get_type_parts()));
    }
}

impl Item {
    pub fn new(id: &str, params: ItemParams) -> Self {
        Item {
            id: id.to_string(),
            name: params.name,
            effects: params.effects,
            uses: params.uses,
            duration: params.duration,
            mount_offset: params.mount_offset,
            drop_behavior: params.drop_behavior,
            deplete_behavior: params.deplete_behavior,
            is_hat: params.is_hat,
            duration_timer: 0.0,
            use_cnt: 0,
        }
    }
}

/// This holds the parameters used when constructing an `Equipment`
#[derive(Clone, Serialize, Deserialize, TypeName)]
pub struct ItemMetadata {
    /// The effects that will be instantiated when the item is equipped
    #[serde(default)]
    pub effects: Vec<PassiveEffectMetadata>,
    /// The items duration, after being equipped. This will also be the default duration of
    /// passive effects that are added to the player, when equipping the item
    #[serde(default)]
    pub duration: Option<f32>,
    /// If this is `true` the item will be treated as a hat
    #[serde(default, rename = "hat", skip_serializing_if = "core::json::is_false")]
    pub is_hat: bool,
}

impl<'lua> FromLua<'lua> for ItemMetadata {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        hv_lua::LuaSerdeExt::from_value(lua, lua_value)
    }
}
impl<'lua> ToLua<'lua> for ItemMetadata {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        hv_lua::LuaSerdeExt::to_value(lua, &self)
    }
}

impl TypeBody for ItemMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("effects").into(),
            Vec::<PassiveEffectMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("duration").into(),
            Option::<f32>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_hat").into(), bool::get_type_parts()));
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapItemMetadata {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub kind: MapItemKind,
    #[serde(with = "core::json::vec2_def")]
    pub collider_size: Vec2,
    #[serde(default, with = "core::json::vec2_def")]
    pub collider_offset: Vec2,
    #[serde(default)]
    pub uses: Option<u32>,
    #[serde(default)]
    pub drop_behavior: ItemDropBehavior,
    #[serde(default)]
    pub deplete_behavior: ItemDepleteBehavior,
    /// This specifies the offset from the player position to where the equipped item is drawn
    #[serde(default, with = "core::json::vec2_def")]
    pub mount_offset: Vec2,
    /// The parameters for the `AnimationPlayer` that will be used to draw the item
    #[serde(alias = "animation")]
    pub sprite: AnimatedSpriteMetadata,
}

pub fn spawn_item(world: &mut World, position: Vec2, meta: MapItemMetadata) -> Result<Entity> {
    let mut sprites = Vec::new();

    let MapItemMetadata {
        collider_size,
        collider_offset,
        drop_behavior,
        deplete_behavior,
        mount_offset,
        ..
    } = meta;

    let actor = storage::get_mut::<CollisionWorld>().add_actor(
        position,
        collider_size.x as i32,
        collider_size.y as i32,
    );

    let animations = meta
        .sprite
        .animations
        .clone()
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<_>>();

    let sprite = AnimatedSprite::new(
        &meta.sprite.texture_id,
        animations.as_slice(),
        meta.sprite.clone().into(),
    );

    sprites.push((SPRITE_ANIMATED_SPRITE_ID, sprite));

    let id = meta.id.as_str();

    let entity = world.spawn((
        Transform::from(position),
        PhysicsBody::new(
            actor,
            None,
            PhysicsBodyParams {
                size: collider_size,
                offset: collider_offset,
                has_mass: true,
                has_friction: true,
                can_rotate: false,
                ..Default::default()
            },
        ),
    ));

    let uses = meta.uses;

    let name = meta.name.clone();

    match meta.kind {
        MapItemKind::Item { meta } => {
            let ItemMetadata {
                effects,
                duration,
                is_hat,
            } = meta;

            world.insert_one(
                entity,
                Item::new(
                    id,
                    ItemParams {
                        name,
                        effects,
                        uses,
                        duration,
                        mount_offset,
                        drop_behavior,
                        deplete_behavior,
                        is_hat,
                    },
                ),
            )?;

            if !sprites.is_empty() {
                world.insert_one(
                    entity,
                    Drawable::new_animated_sprite_set(ITEMS_DRAW_ORDER, sprites.as_slice()),
                )?;
            }
        }
        MapItemKind::Weapon { meta } => {
            let effect_offset = meta.effect_offset;

            let mut sound_effect = None;
            if let Some(id) = meta.sound_effect_id.as_ref() {
                sound_effect = storage::get::<Resources>().sounds.get(id).copied();
            }

            if let Some(effect_sprite) = meta.effect_sprite {
                let animations = effect_sprite
                    .animations
                    .clone()
                    .into_iter()
                    .map(|a| a.into())
                    .collect::<Vec<_>>();

                let mut sprite = AnimatedSprite::new(
                    &effect_sprite.texture_id,
                    animations.as_slice(),
                    effect_sprite.clone().into(),
                );

                sprite.is_deactivated = true;

                sprites.push((EFFECT_ANIMATED_SPRITE_ID, sprite));
            }

            let particle_emitters = meta
                .particles
                .clone()
                .into_iter()
                .map(ParticleEmitter::new)
                .collect::<Vec<_>>();

            if !particle_emitters.is_empty() {
                world.insert_one(entity, particle_emitters).unwrap();
            }

            let params = WeaponParams {
                name,
                effects: meta.effects,
                uses,
                sound_effect,
                mount_offset,
                effect_offset,
                drop_behavior,
                deplete_behavior,
            };

            world.insert_one(
                entity,
                Weapon::new(id, meta.recoil, meta.cooldown, meta.attack_duration, params),
            )?;

            if !sprites.is_empty() {
                world.insert_one(
                    entity,
                    Drawable::new_animated_sprite_set(ITEMS_DRAW_ORDER, sprites.as_slice()),
                )?;
            }
        }
    }

    Ok(entity)
}

pub struct WeaponParams {
    pub name: String,
    pub effects: Vec<ActiveEffectMetadata>,
    pub uses: Option<u32>,
    pub sound_effect: Option<Sound>,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
}

impl Default for WeaponParams {
    fn default() -> Self {
        WeaponParams {
            name: "".to_string(),
            effects: Vec::new(),
            uses: None,
            sound_effect: None,
            mount_offset: Vec2::ZERO,
            effect_offset: Vec2::ZERO,
            drop_behavior: Default::default(),
            deplete_behavior: Default::default(),
        }
    }
}

#[derive(Clone, TypeName)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub effects: Vec<ActiveEffectMetadata>,
    pub sound_effect: Option<Sound>,
    pub recoil: f32,
    pub cooldown: f32,
    pub attack_duration: f32,
    pub uses: Option<u32>,
    pub mount_offset: Vec2,
    pub effect_offset: Vec2,
    pub drop_behavior: ItemDropBehavior,
    pub deplete_behavior: ItemDepleteBehavior,
    pub cooldown_timer: f32,
    pub use_cnt: u32,
}

impl<'lua> FromLua<'lua> for Weapon {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            id: table.get("id")?,
            name: table.get("name")?,
            effects: table.get("effects")?,
            sound_effect: table
                .get::<_, Option<SoundLua>>("sound_effect")?
                .map(From::from),
            recoil: table.get("recoil")?,
            cooldown: table.get("cooldown")?,
            attack_duration: table.get("attack_duration")?,
            uses: table.get("uses")?,
            mount_offset: table.get::<_, Vec2Lua>("mount_offset")?.into(),
            effect_offset: table.get::<_, Vec2Lua>("effect_offset")?.into(),
            drop_behavior: table.get("drop_behavior")?,
            deplete_behavior: table.get("deplete_behavior")?,
            cooldown_timer: table.get("cooldown_timer")?,
            use_cnt: table.get("use_cnt")?,
        })
    }
}

impl<'lua> ToLua<'lua> for Weapon {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("name", self.name)?;
        table.set("effects", self.effects)?;
        table.set("sound_effect", self.sound_effect.map(SoundLua::from))?;
        table.set("recoil", self.recoil)?;
        table.set("cooldown", self.cooldown)?;
        table.set("attack_duration", self.attack_duration)?;
        table.set("uses", self.uses)?;
        table.set("mount_offset", Vec2Lua::from(self.mount_offset))?;
        table.set("effect_offset", Vec2Lua::from(self.effect_offset))?;
        table.set("drop_behavior", self.drop_behavior)?;
        table.set("deplete_behavior", self.deplete_behavior)?;
        table.set("cooldown_timer", self.cooldown_timer)?;
        table.set("use_cnt", self.use_cnt)?;
        lua.pack(table)
    }
}

impl TypeBody for Weapon {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("id").into(), String::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("name").into(), String::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("effects").into(),
            Vec::<ActiveEffectMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("sound_effect").into(),
            Option::<SoundLua>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("recoil").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("cooldown").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("attack_duration").into(),
            f32::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("uses").into(),
            Option::<u32>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("mount_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("effect_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("drop_behavior").into(),
            ItemDropBehavior::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("deplete_behavior").into(),
            ItemDepleteBehavior::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("cooldown_timer").into(),
            f32::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("use_cnt").into(), u32::get_type_parts()));
    }
}

impl Weapon {
    pub fn new(
        id: &str,
        recoil: f32,
        cooldown: f32,
        attack_duration: f32,
        params: WeaponParams,
    ) -> Self {
        Weapon {
            id: id.to_string(),
            name: params.name,
            effects: params.effects,
            recoil,
            cooldown,
            uses: params.uses,
            attack_duration,
            sound_effect: params.sound_effect,
            mount_offset: params.mount_offset,
            effect_offset: params.effect_offset,
            drop_behavior: params.drop_behavior,
            deplete_behavior: params.deplete_behavior,
            cooldown_timer: cooldown,
            use_cnt: 0,
        }
    }
}

pub fn fire_weapon(world: &mut World, entity: Entity, owner: Entity) -> Result<()> {
    let mut effects = Vec::new();

    let mut origin = Vec2::ZERO;

    {
        let mut weapon = world.get_mut::<Weapon>(entity).unwrap();

        if weapon.cooldown_timer >= weapon.cooldown {
            let mut player = world.get_mut::<Player>(owner).unwrap();

            {
                let mut owner_body = world.get_mut::<PhysicsBody>(owner).unwrap();

                if player.is_facing_left {
                    owner_body.velocity.x = weapon.recoil;
                } else {
                    owner_body.velocity.x = -weapon.recoil;
                }

                let owner_transform = world.get::<Transform>(owner).unwrap();
                let owner_inventory = world.get::<PlayerInventory>(owner).unwrap();

                origin = owner_transform.position
                    + owner_inventory
                        .get_weapon_mount(player.is_facing_left, player.is_upside_down);

                let mut offset = weapon.mount_offset + weapon.effect_offset;
                if player.is_facing_left {
                    offset.x = -offset.x;
                }

                origin += offset;
            }

            player.attack_timer = weapon.attack_duration;

            weapon.use_cnt += 1;

            weapon.cooldown_timer = 0.0;

            if let Some(sound) = weapon.sound_effect {
                play_sound_once(sound);
            }

            let mut drawable = world.get_mut::<Drawable>(entity).unwrap();
            {
                let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

                {
                    let sprite = sprite_set.map.get_mut(SPRITE_ANIMATED_SPRITE_ID).unwrap();
                    let is_looping = sprite
                        .get_animation(ATTACK_ANIMATION_ID)
                        .map(|a| a.is_looping)
                        .unwrap_or_default();

                    sprite.set_animation(ATTACK_ANIMATION_ID, !is_looping);
                    sprite.queue_action(QueuedAnimationAction::Play(IDLE_ANIMATION_ID.to_string()));
                }

                if let Some(sprite) = sprite_set.map.get_mut(EFFECT_ANIMATED_SPRITE_ID) {
                    sprite.is_deactivated = false;

                    let is_looping = sprite
                        .get_animation(ATTACK_ANIMATION_ID)
                        .map(|a| a.is_looping)
                        .unwrap_or_default();

                    sprite.set_animation(ATTACK_ANIMATION_ID, !is_looping);
                    sprite.queue_action(QueuedAnimationAction::Deactivate);
                }
            }

            if let Ok(mut particle_emitters) = world.get_mut::<Vec<ParticleEmitter>>(entity) {
                for emitter in particle_emitters.iter_mut() {
                    emitter.activate();
                }
            }

            effects = weapon.effects.clone();
        }
    }

    for params in effects {
        spawn_active_effect(world, owner, origin, params)?;
    }

    Ok(())
}

/// This holds the parameters for the `AnimationPlayer` components of an equipped `Weapon`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponAnimationMetadata {
    /// This holds the parameters of the main `AnimationPlayer` component, holding the main
    /// animations, like `"idle"` and `"attack"`.
    /// At a minimum, an animation with the id `"idle"` must be specified. If no animation is
    /// required, an animation with one frame can be used to just display a sprite.
    #[serde(rename = "animation")]
    pub sprite: AnimatedSpriteMetadata,
    /// This can hold the parameters of the effect `AnimationPlayer` component, holding the
    /// animations used for effects like `"attack_effect"`.
    /// At a minimum, if this is specified, an animation with the id `"attack_effect"` must be
    /// specified. If no animation is required, an animation with one frame can be used to just
    /// display a sprite.
    #[serde(
        default,
        rename = "effect_animation",
        skip_serializing_if = "Option::is_none"
    )]
    pub effect: Option<AnimatedSpriteMetadata>,
}

/// This holds parameters specific to the `Weapon` variant of `ItemKind`, used to instantiate a
/// `Weapon` struct instance, when an `Item` of type `Weapon` is picked up.
#[derive(Clone, Serialize, Deserialize)]
pub struct WeaponMetadata {
    /// This specifies the effects to instantiate when the weapon is used to attack
    #[serde(default)]
    pub effects: Vec<ActiveEffectMetadata>,
    /// Particle effects that will be activated when using the weapon
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub particles: Vec<ParticleEmitterMetadata>,
    /// This can specify an id of a sound effect that is played when the weapon is used to attack
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    /// This specifies the offset between the upper left corner of the weapon's sprite to the
    /// position that will serve as the origin of the weapon's effects
    #[serde(default, with = "core::json::vec2_def")]
    pub effect_offset: Vec2,
    /// This can specify a maximum amount of weapon uses. If no value is specified, the weapon
    /// will have unlimited uses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This specifies the minimum interval of attacks with the weapon
    #[serde(default)]
    pub cooldown: f32,
    /// This specifies the amount of time the player will be locked in an attack state when using
    /// the weapon
    #[serde(default)]
    pub attack_duration: f32,
    /// This specifies the force applied to the `Player` velocity, in the opposite direction of the
    /// attack, when the weapon is activated.
    #[serde(default)]
    pub recoil: f32,
    /// This can hold the parameters of the effect `AnimationPlayer` component, holding the
    /// animations used for effects.
    /// At a minimum, if this is specified, an animation with the id `"attack"` must be
    /// specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_sprite: Option<AnimatedSpriteMetadata>,
}

impl<'lua> FromLua<'lua> for WeaponMetadata {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        hv_lua::LuaSerdeExt::from_value(lua, lua_value)
    }
}

impl<'lua> ToLua<'lua> for WeaponMetadata {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        hv_lua::LuaSerdeExt::to_value(lua, &self)
    }
}

impl TypeBody for WeaponMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("effects").into(),
            Vec::<ActiveEffectMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("particles").into(),
            Vec::<ParticleEmitterMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("sound_effect_id").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("effect_offset").into(),
            Vec2Lua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("uses").into(),
            Option::<u32>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("cooldown").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("attack_duration").into(),
            f32::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("recoil").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("effect_sprite").into(),
            Option::<AnimatedSpriteMetadata>::get_type_parts(),
        ));
    }
}

impl Default for WeaponMetadata {
    fn default() -> Self {
        WeaponMetadata {
            effects: Vec::new(),
            particles: Vec::new(),
            sound_effect_id: None,
            uses: None,
            effect_offset: Vec2::ZERO,
            cooldown: 0.0,
            attack_duration: 0.0,
            recoil: 0.0,
            effect_sprite: None,
        }
    }
}
