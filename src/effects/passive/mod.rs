use std::borrow::Cow;
use std::collections::HashMap;

use hv_lua::{FromLua, ToLua};
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use hecs::{Entity, World};
use tealr::{TypeBody, TypeName};

mod turtle_shell;

use crate::player::PlayerEventKind;
use crate::PlayerEvent;

static mut PASSIVE_EFFECT_FUNCS: Option<HashMap<String, PassiveEffectFn>> = None;

unsafe fn get_passive_effects_map() -> &'static mut HashMap<String, PassiveEffectFn> {
    PASSIVE_EFFECT_FUNCS.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
pub fn add_passive_effect(id: &str, f: PassiveEffectFn) {
    unsafe { get_passive_effects_map() }.insert(id.to_string(), f);
}

pub fn try_get_passive_effect(id: &str) -> Option<&PassiveEffectFn> {
    unsafe { get_passive_effects_map() }.get(id)
}

pub fn get_passive_effect(id: &str) -> &PassiveEffectFn {
    try_get_passive_effect(id).unwrap()
}

pub type PassiveEffectFn =
    fn(world: &mut World, player_entity: Entity, item_entity: Option<Entity>, event: PlayerEvent);

pub fn init_passive_effects() {
    let effects = unsafe { get_passive_effects_map() };

    effects.insert(
        turtle_shell::EFFECT_FUNCTION_ID.to_string(),
        turtle_shell::effect_function,
    );
}

pub struct PassiveEffectInstance {
    pub name: String,
    pub function: Option<PassiveEffectFn>,
    pub activated_on: Vec<PlayerEventKind>,
    pub particle_effect_id: Option<String>,
    pub event_particle_effect_id: Option<String>,
    pub blocks_damage: bool,
    pub uses: Option<u32>,
    pub item: Option<Entity>,
    pub use_cnt: u32,
    pub duration: Option<f32>,
    pub duration_timer: f32,
}

impl PassiveEffectInstance {
    pub fn new(item: Option<Entity>, meta: PassiveEffectMetadata) -> Self {
        let function = meta.function_id.map(|id| *get_passive_effect(&id));

        PassiveEffectInstance {
            name: meta.name,
            function,
            activated_on: meta.activated_on,
            particle_effect_id: meta.particle_effect_id,
            event_particle_effect_id: meta.event_particle_effect_id,
            blocks_damage: meta.blocks_damage,
            uses: meta.uses,
            item,
            use_cnt: 0,
            duration: meta.duration,
            duration_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.duration_timer += dt;
    }

    pub fn is_depleted(&self) -> bool {
        if let Some(duration) = self.duration {
            if self.duration_timer >= duration {
                return true;
            }
        }

        if let Some(uses) = self.uses {
            if self.use_cnt >= uses {
                return true;
            }
        }

        false
    }
}

#[derive(Clone, Serialize, Deserialize, TypeName)]
pub struct PassiveEffectMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_id: Option<String>,
    /// This specifies the player events that will trigger an activation of the event
    pub activated_on: Vec<PlayerEventKind>,
    /// This is the particle effect that will be spawned when the effect become active.
    #[serde(
        default,
        rename = "particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub particle_effect_id: Option<String>,
    /// This is the particle effect that will be spawned, each time a player event leads to the
    /// effect coroutine being called.
    #[serde(
        default,
        rename = "event_particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub event_particle_effect_id: Option<String>,
    /// If this is true damage will be blocked on a player that has the item equipped
    #[serde(default)]
    pub blocks_damage: bool,
    /// This is the amount of times the coroutine can be called, before the effect is depleted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This is the duration of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}

impl<'lua> FromLua<'lua> for PassiveEffectMetadata {
    fn from_lua(lua_value: hv_lua::Value<'lua>, lua: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        hv_lua::LuaSerdeExt::from_value(lua, lua_value)
    }
}

impl<'lua> ToLua<'lua> for PassiveEffectMetadata {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        hv_lua::LuaSerdeExt::to_value(lua, &self)
    }
}

impl TypeBody for PassiveEffectMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("name"),
            tealr::type_parts_to_str(String::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("function_id"),
            tealr::type_parts_to_str(Option::<String>::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("activated_on"),
            tealr::type_parts_to_str(Vec::<PlayerEventKind>::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("particle_effect"),
            tealr::type_parts_to_str(Option::<String>::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("event_particle_effect"),
            tealr::type_parts_to_str(Option::<String>::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("blocks_damage"),
            tealr::type_parts_to_str(bool::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("uses"),
            tealr::type_parts_to_str(Option::<u32>::get_type_parts()),
        ));
        gen.fields.push((
            Cow::Borrowed("duration"),
            tealr::type_parts_to_str(Option::<f32>::get_type_parts()),
        ));
    }
}
