use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, Function, RegistryKey, Table, ToLua, UserData};
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use hecs::{Entity, World};
use tealr::mlu::{TealData, TypedFunction, UserDataWrapper};
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

#[derive(Clone)]
pub enum PassiveEffectFnContainer {
    SimpleFn(PassiveEffectFn),
    ///we are going to keep the functions inside the lua vm. So, this is just a way to find the function back
    LuaFnPointer(Arc<RegistryKey>),
}

impl From<PassiveEffectFn> for PassiveEffectFnContainer {
    fn from(value: PassiveEffectFn) -> Self {
        Self::SimpleFn(value)
    }
}

impl PassiveEffectFnContainer {
    pub fn call_get_lua(
        &self,
        world: Arc<AtomicRefCell<World>>,
        player_entity: Entity,
        item_entity: Option<Entity>,
        event: PlayerEvent,
    ) -> hv_lua::Result<()> {
        match self {
            PassiveEffectFnContainer::SimpleFn(x) => {
                x(&mut world.borrow_mut(), player_entity, item_entity, event);
                Ok(())
            }
            PassiveEffectFnContainer::LuaFnPointer(_) => {
                todo!("Tried calling a PassiveEffectFnContainer::LuaFnPointer. This should not yet be possible")
            }
        }
    }
    pub fn call(
        &self,
        lua: &hv_lua::Lua,
        world: Arc<AtomicRefCell<World>>,
        player_entity: Entity,
        item_entity: Option<Entity>,
        event: PlayerEvent,
    ) -> hv_lua::Result<()> {
        match self {
            PassiveEffectFnContainer::SimpleFn(x) => {
                x(&mut world.borrow_mut(), player_entity, item_entity, event);
                Ok(())
            }
            PassiveEffectFnContainer::LuaFnPointer(key) => {
                let data = lua.registry_value::<Table>(key)?;
                let globals = lua.globals();
                let old_require = globals.get::<_, Option<Function>>("require")?;
                let new_require = data.get::<_, Function>("require")?;
                let function = data.get::<_, TypedFunction<
                    (
                        Arc<AtomicRefCell<World>>,
                        Entity,
                        Option<Entity>,
                        PlayerEvent,
                    ),
                    (),
                >>("function")?;
                globals.set("require", new_require)?;
                let res = function.call((world, player_entity, item_entity, event));
                //if this fails then the `require` function isn't the same one as which we started with
                //this could be a problem, however I think it is unlikely to happen as this value comes from the lua instance already.
                //Also, if the value of `old_require` is None then this doesn't matter.
                globals.set("require", old_require)?;
                res
            }
        }
    }
}

impl TypeName for PassiveEffectFnContainer {
    fn get_type_parts() -> Cow<'static, [tealr::NamePart]> {
        //this is a lie
        //however, I want to make it so the user doesn't need to know about this thing not being a function
        //right now, it already acts like one as you can just call it on the lua side
        //once https://github.com/sdleffler/hv-lua/pull/2 is merged, it should also be possible that it can be created by a normal function
        //this should make it even more similar.

        //after that, the only thing that should be different is how it converts to a string, and the result of the typeof function
        //both of those don't have a good solution, but only the `typeof` thing might be relevant.
        tealr::mlu::TypedFunction::<(World,Entity,Option<Entity>,PlayerEvent),()>::get_type_parts()
    }
}

impl UserData for PassiveEffectFnContainer {
    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
}

impl TealData for PassiveEffectFnContainer {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(
            tealr::mlu::mlua::MetaMethod::Call,
            |lua, this, (world, player_entity, item_entity, event)| {
                this.call(lua, world, player_entity, item_entity, event)
            },
        )
    }
}

pub fn init_passive_effects() {
    let effects = unsafe { get_passive_effects_map() };

    effects.insert(
        turtle_shell::EFFECT_FUNCTION_ID.to_string(),
        turtle_shell::effect_function,
    );
}

#[derive(Clone, TypeName)]
pub struct PassiveEffectInstance {
    pub name: String,
    pub function: Option<PassiveEffectFnContainer>,
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

impl UserData for PassiveEffectInstance {
    fn add_methods<'lua, M: hv_lua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
    fn add_fields<'lua, F: hv_lua::UserDataFields<'lua, Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper)
    }
}
impl TealData for PassiveEffectInstance {
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.to_owned()));
        fields.add_field_method_get("function", |_, this| Ok(this.function.to_owned()));
        fields.add_field_method_get("activated_on", |_, this| Ok(this.activated_on.to_owned()));
        fields.add_field_method_get("particle_effect_id", |_, this| {
            Ok(this.particle_effect_id.to_owned())
        });
        fields.add_field_method_get("event_particle_effect_id", |_, this| {
            Ok(this.event_particle_effect_id.to_owned())
        });
        fields.add_field_method_get("blocks_damage", |_, this| Ok(this.blocks_damage));
        fields.add_field_method_get("uses", |_, this| Ok(this.uses));
        fields.add_field_method_get("item", |_, this| Ok(this.item));
        fields.add_field_method_get("use_cnt", |_, this| Ok(this.use_cnt));
        fields.add_field_method_get("duration", |_, this| Ok(this.duration));
        fields.add_field_method_get("duration_timer", |_, this| Ok(this.duration_timer));
        fields.add_field_method_set("name", |_, this, value| {
            this.name = value;
            Ok(())
        });
        fields.add_field_method_set("function", |_, this, value| {
            this.function = value;
            Ok(())
        });
        fields.add_field_method_set("activated_on", |_, this, value| {
            this.activated_on = value;
            Ok(())
        });
        fields.add_field_method_set("particle_effect_id", |_, this, value| {
            this.particle_effect_id = value;
            Ok(())
        });
        fields.add_field_method_set("event_particle_effect_id", |_, this, value| {
            this.event_particle_effect_id = value;
            Ok(())
        });
        fields.add_field_method_set("blocks_damage", |_, this, value| {
            this.blocks_damage = value;
            Ok(())
        });
        fields.add_field_method_set("uses", |_, this, value| {
            this.uses = value;
            Ok(())
        });
        fields.add_field_method_set("item", |_, this, value| {
            this.item = value;
            Ok(())
        });
        fields.add_field_method_set("use_cnt", |_, this, value| {
            this.use_cnt = value;
            Ok(())
        });
        fields.add_field_method_set("duration", |_, this, value| {
            this.duration = value;
            Ok(())
        });
        fields.add_field_method_set("duration_timer", |_, this, value| {
            this.duration_timer = value;
            Ok(())
        });
    }
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut("update", |_, this, dt| {
            this.update(dt);
            Ok(())
        });
        methods.add_method("is_depleted", |_, this, ()| Ok(this.is_depleted()));
    }
}

impl TypeBody for PassiveEffectInstance {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_fields(gen);
        <Self as TealData>::add_methods(gen);
    }
}

impl PassiveEffectInstance {
    pub fn new(item: Option<Entity>, meta: PassiveEffectMetadata) -> Self {
        let function = meta.function_id.map(|id| (*get_passive_effect(&id)).into());

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
        gen.fields
            .push((Cow::Borrowed("name").into(), String::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("function_id").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("activated_on").into(),
            Vec::<PlayerEventKind>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("particle_effect").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("event_particle_effect").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("blocks_damage").into(),
            bool::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("uses").into(),
            Option::<u32>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("duration").into(),
            Option::<f32>::get_type_parts(),
        ));
    }
}
