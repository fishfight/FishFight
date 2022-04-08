use hv_alchemy::Type;
use hv_cell::AtomicRefCell;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use mlua::{FromLua, ToLua, UserData, UserDataMethods};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tealr::mlu::{MaybeSend, UserDataWrapper};
use tealr::TypeName;
use tealr::{mlu::TealData, TypeBody};

use ff_particles::EmittersCache;

use hecs::World;

use serde::{Deserialize, Serialize};

use core::Transform;
use core::{lua::wrapped_types::Vec2Lua, math::IsZero};

use crate::{AnimatedSpriteMetadata, Resources};

#[derive(Clone, Debug, Serialize, Deserialize, tealr::TypeName)]
pub struct ParticleEmitterMetadata {
    /// The id of the particle effect.
    #[serde(rename = "particle_effect")]
    pub particle_effect_id: String,
    /// The offset is added to the `position` provided when calling `draw`
    #[serde(
        default,
        with = "core::json::vec2_def",
        skip_serializing_if = "Vec2::is_zero"
    )]
    pub offset: Vec2,
    /// Delay before emission will begin
    #[serde(default, skip_serializing_if = "f32::is_zero")]
    pub delay: f32,
    /// The interval between each emission.
    #[serde(default, skip_serializing_if = "f32::is_zero")]
    pub interval: f32,
    /// Amount of emissions per activation. If set to `None` it will emit indefinitely
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissions: Option<u32>,
    /// This is a temporary hack that enables texture based effects until we add texture support
    /// to our macroquad-particles fork
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animations: Option<AnimatedSpriteMetadata>,
    /// If this is set to `true` the `ParticleController` will start to emit automatically
    #[serde(default, skip_serializing_if = "core::json::is_false")]
    pub should_autostart: bool,
}
impl TypeBody for ParticleEmitterMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("particle_effect").into(),
            String::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("delay").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("interval").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("emissions").into(),
            Option::<u32>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("animations").into(),
            Option::<AnimatedSpriteMetadata>::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("should_autostart").into(),
            bool::get_type_parts(),
        ));
    }
}

impl<'lua> FromLua<'lua> for ParticleEmitterMetadata {
    fn from_lua(lua_value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        mlua::LuaSerdeExt::from_value(lua, lua_value)
    }
}

impl<'lua> ToLua<'lua> for ParticleEmitterMetadata {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        mlua::LuaSerdeExt::to_value(lua, &self)
    }
}

impl Default for ParticleEmitterMetadata {
    fn default() -> Self {
        ParticleEmitterMetadata {
            particle_effect_id: "".to_string(),
            offset: Vec2::ZERO,
            delay: 0.0,
            emissions: None,
            interval: 0.0,
            animations: None,
            should_autostart: false,
        }
    }
}
use hv_lua as mlua;

#[derive(Clone, tealr::TypeName)]
pub struct ParticleEmitter {
    pub particle_effect_id: String,
    pub offset: Vec2,
    pub delay: f32,
    pub emissions: Option<u32>,
    pub interval: f32,
    pub emission_cnt: u32,
    pub delay_timer: f32,
    pub interval_timer: f32,
    pub is_active: bool,
}

impl ParticleEmitter {
    pub fn new(meta: ParticleEmitterMetadata) -> Self {
        ParticleEmitter {
            particle_effect_id: meta.particle_effect_id,
            offset: meta.offset,
            delay: meta.delay,
            interval: meta.interval,
            emissions: meta.emissions,
            emission_cnt: 0,
            delay_timer: 0.0,
            interval_timer: meta.interval,
            is_active: meta.should_autostart,
        }
    }

    pub fn get_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
        let mut offset = self.offset;

        if flip_x {
            offset.x = -offset.x;
        }

        if flip_y {
            offset.y = -offset.y;
        }

        offset
    }

    pub fn activate(&mut self) {
        self.delay_timer = 0.0;
        self.interval_timer = self.interval;
        self.emission_cnt = 0;
        self.is_active = true;
    }
}

impl UserData for ParticleEmitter {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper)
    }

    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
    fn add_type_methods<'lua, M: UserDataMethods<'lua, Type<Self>>>(methods: &mut M)
    where
        Self: 'static + MaybeSend,
    {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_type_methods(&mut wrapper)
    }
}
impl TypeBody for ParticleEmitter {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_fields(gen);
        <Self as TealData>::add_methods(gen);
    }
    fn get_type_body_marker(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_type_methods(gen);
    }
}

impl TealData for ParticleEmitter {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("get_offset", |lua, this, (flip_x, flip_y): (bool, bool)| {
            Ok(Vec2Lua::from(this.get_offset(flip_x, flip_y)))
        });
        methods.add_method_mut("activate", |_, this, ()| {
            this.activate();
            Ok(())
        })
    }
    fn add_type_methods<'lua, M: tealr::mlu::TealDataMethods<'lua, Type<Self>>>(methods: &mut M)
    where
        Self: 'static + tealr::mlu::MaybeSend,
    {
        methods.add_function("new", |_, meta| Ok(ParticleEmitter::new(meta)))
    }
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("particle_effect_id", |lua, this| {
            Ok(this.particle_effect_id.clone())
        });
        fields.add_field_method_get("offset", |lua, this| Ok(Vec2Lua::from(this.offset)));
        fields.add_field_method_get("delay", |lua, this| Ok(this.delay));
        fields.add_field_method_get("emissions", |lua, this| Ok(this.emissions));
        fields.add_field_method_get("interval", |lua, this| Ok(this.interval));
        fields.add_field_method_get("emission_cnt", |lua, this| Ok(this.emission_cnt));
        fields.add_field_method_get("delay_timer", |lua, this| Ok(this.delay_timer));
        fields.add_field_method_get("interval_timer", |lua, this| Ok(this.interval_timer));
        fields.add_field_method_get("is_active", |lua, this| Ok(this.is_active));

        fields.add_field_method_set("particle_effect_id", |_, this, value| {
            this.particle_effect_id = value;
            Ok(())
        });
        fields.add_field_method_set("offset", |_, this, value: Vec2Lua| {
            this.offset = value.into();
            Ok(())
        });
        fields.add_field_method_set("delay", |_, this, value| {
            this.delay = value;
            Ok(())
        });
        fields.add_field_method_set("emissions", |_, this, value| {
            this.emissions = value;
            Ok(())
        });
        fields.add_field_method_set("interval", |_, this, value| {
            this.interval = value;
            Ok(())
        });
        fields.add_field_method_set("emission_cnt", |_, this, value| {
            this.emission_cnt = value;
            Ok(())
        });
        fields.add_field_method_set("delay_timer", |_, this, value| {
            this.delay_timer = value;
            Ok(())
        });
        fields.add_field_method_set("interval_timer", |_, this, value| {
            this.interval_timer = value;
            Ok(())
        });
        fields.add_field_method_set("is_active", |_, this, value| {
            this.is_active = value;
            Ok(())
        });
    }
}

impl From<ParticleEmitterMetadata> for ParticleEmitter {
    fn from(params: ParticleEmitterMetadata) -> Self {
        ParticleEmitter::new(params)
    }
}

pub fn update_one_particle_emitter(
    mut position: Vec2,
    rotation: f32,
    emitter: &mut ParticleEmitter,
) {
    let dt = get_frame_time();

    if emitter.is_active {
        emitter.delay_timer += dt;

        if emitter.delay_timer >= emitter.delay {
            emitter.interval_timer += dt;
        }

        if emitter.delay_timer >= emitter.delay && emitter.interval_timer >= emitter.interval {
            emitter.interval_timer = 0.0;

            if rotation == 0.0 {
                position += emitter.offset;
            } else {
                let offset_position = position + emitter.offset;

                let sin = rotation.sin();
                let cos = rotation.cos();

                position = Vec2::new(
                    cos * (offset_position.x - position.x) - sin * (offset_position.y - position.y)
                        + position.x,
                    sin * (offset_position.x - position.x)
                        + cos * (offset_position.y - position.y)
                        + position.y,
                );
            }

            let mut particles = storage::get_mut::<Particles>();
            let cache = particles
                .cache_map
                .get_mut(&emitter.particle_effect_id)
                .unwrap();

            cache.spawn(position);

            if let Some(emissions) = emitter.emissions {
                emitter.emission_cnt += 1;

                if emissions > 0 && emitter.emission_cnt >= emissions {
                    emitter.is_active = false;
                }
            }
        }
    }
}

pub fn update_particle_emitters(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, (transform, emitter)) in world.query_mut::<(&Transform, &mut ParticleEmitter)>() {
        update_one_particle_emitter(transform.position, transform.rotation, emitter);
    }

    for (_, (transform, emitters)) in world.query_mut::<(&Transform, &mut Vec<ParticleEmitter>)>() {
        for emitter in emitters.iter_mut() {
            update_one_particle_emitter(transform.position, transform.rotation, emitter);
        }
    }
}

pub fn draw_particles(_world: Arc<AtomicRefCell<World>>) {
    let mut particles = storage::get_mut::<Particles>();

    for cache in particles.cache_map.values_mut() {
        cache.draw();
    }
}

#[derive(Default)]
pub struct Particles {
    pub cache_map: HashMap<String, EmittersCache>,
}

impl Particles {
    pub fn new() -> Self {
        let mut cache_map = HashMap::new();

        let resources = storage::get::<Resources>();

        for id in resources.particle_effects.keys() {
            let config = resources.particle_effects.get(id).cloned().unwrap();

            cache_map.insert(id.clone(), EmittersCache::new(config));
        }

        Particles { cache_map }
    }
}
