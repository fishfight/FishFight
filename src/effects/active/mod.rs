use hecs::{Entity, World};
use hv_cell::AtomicRefCell;
use macroquad::audio::play_sound_once;
use macroquad::color;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use mlua::{FromLua, ToLua};
use serde::{Deserialize, Serialize};
use tealr::mlu::TealData;

use core::math::{deg_to_rad, rotate_vector, IsZero};
use core::Result;
use std::borrow::Cow;
use std::sync::Arc;

use crate::Resources;
use crate::{PassiveEffectInstance, PassiveEffectMetadata};

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectMetadata, TriggeredEffectTrigger};

use crate::effects::active::projectiles::{spawn_projectile, ProjectileParams};
use crate::effects::active::triggered::{spawn_triggered_effect, TriggeredEffect};
use crate::particles::ParticleEmitterMetadata;
use crate::player::{on_player_damage, Player};
use crate::PhysicsBody;
use core::Transform;
pub use projectiles::ProjectileKind;

const COLLIDER_DEBUG_DRAW_TTL: f32 = 0.5;

use hv_lua as mlua;
use tealr::{MluaTealDerive, TypeBody, TypeName};
#[derive(Clone, MluaTealDerive)]
struct CircleCollider {
    r: f32,
    ttl_timer: f32,
}

impl TealData for CircleCollider {}

#[derive(Clone, MluaTealDerive)]
struct RectCollider {
    w: f32,
    h: f32,
    ttl_timer: f32,
}

impl TealData for RectCollider {}

pub fn spawn_active_effect(
    world: &mut World,
    owner: Entity,
    origin: Vec2,
    params: ActiveEffectMetadata,
) -> Result<()> {
    let is_facing_left = {
        let player = world.get::<Player>(owner).unwrap();
        player.is_facing_left
    };

    if let Some(id) = &params.sound_effect_id {
        let resources = storage::get::<Resources>();
        let sound = resources.sounds.get(id).unwrap();

        play_sound_once(*sound);
    }

    let mut damage = Vec::new();

    match *params.kind {
        ActiveEffectKind::CircleCollider {
            radius,
            passive_effects,
            is_lethal,
            is_explosion,
        } => {
            let circle = Circle::new(origin.x, origin.y, radius);

            #[cfg(debug_assertions)]
            {
                world.spawn((
                    Transform::new(origin, 0.0),
                    CircleCollider {
                        r: radius,
                        ttl_timer: 0.0,
                    },
                ));
            }

            for (e, (transform, body)) in world.query::<(&Transform, &PhysicsBody)>().iter() {
                let other_rect = body.as_rect(transform.position);
                if circle.overlaps_rect(&other_rect) {
                    if let Ok(mut player) = world.get_mut::<Player>(e) {
                        if is_explosion || e != owner {
                            if is_lethal {
                                damage.push((owner, e));
                            }

                            for meta in passive_effects.clone().into_iter() {
                                let effect_instance = PassiveEffectInstance::new(None, meta);
                                player.passive_effects.push(effect_instance);
                            }
                        }
                    } else if is_explosion {
                        if let Ok(mut effect) = world.get_mut::<TriggeredEffect>(e) {
                            if effect.trigger.contains(&TriggeredEffectTrigger::Explosion) {
                                effect.is_triggered = true;
                                effect.triggered_by = Some(owner);
                                effect.should_override_delay = true;
                            }
                        }
                    }
                }
            }
        }
        ActiveEffectKind::RectCollider {
            width,
            height,
            is_lethal,
            passive_effects,
        } => {
            let mut rect = Rect::new(origin.x, origin.y, width, height);
            if is_facing_left {
                rect.x -= rect.w;
            }

            #[cfg(debug_assertions)]
            {
                world.spawn((
                    Transform::new(origin, 0.0),
                    RectCollider {
                        w: rect.w,
                        h: rect.h,
                        ttl_timer: 0.0,
                    },
                ));
            }

            for (e, (transform, player, body)) in
                world.query_mut::<(&Transform, &mut Player, &PhysicsBody)>()
            {
                if owner != e {
                    let other_rect = body.as_rect(transform.position);
                    if rect.overlaps(&other_rect) {
                        if is_lethal {
                            damage.push((owner, e));
                        }

                        for meta in passive_effects.clone().into_iter() {
                            let effect_instance = PassiveEffectInstance::new(None, meta);
                            player.passive_effects.push(effect_instance);
                        }
                    }
                }
            }
        }
        ActiveEffectKind::TriggeredEffect { meta } => {
            spawn_triggered_effect(world, owner, origin, is_facing_left, *meta)?;
        }
        ActiveEffectKind::Projectile {
            kind,
            speed,
            range,
            spread,
            is_lethal,
            passive_effects,
            particles,
        } => {
            let mut velocity = Vec2::ZERO;
            if is_facing_left {
                velocity.x = -speed
            } else {
                velocity.x = speed
            }

            if spread != 0.0 {
                let rad = deg_to_rad(spread);
                let spread = rand::gen_range(-rad, rad);

                velocity = rotate_vector(velocity, spread);
            }

            spawn_projectile(
                world,
                owner,
                kind,
                origin,
                velocity,
                range,
                ProjectileParams {
                    is_lethal,
                    passive_effects,
                    particle_effects: particles,
                },
            );
        }
    }

    for (damage_from_entity, damage_to_entity) in damage.drain(0..) {
        on_player_damage(world, damage_from_entity, damage_to_entity);
    }

    Ok(())
}

/// This holds all the common parameters, available to all implementations, as well as specialized
/// parameters, in the `ActiveEffectKind`.
#[derive(Clone, Serialize, Deserialize, TypeName)]
pub struct ActiveEffectMetadata {
    /// This holds all the specialized parameters for the effect, dependent on the implementation,
    /// specified by its variant. It is flattened into this struct in JSON.
    #[serde(flatten)]
    pub kind: Box<ActiveEffectKind>,
    /// This specifies the id of a sound effect to play when the effect is instantiated.
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    /// The delay between instantiation of the effect is requested and the actual instantiation.
    /// This will delay the entire effect, including sound effects, particle effects and anything
    /// drawn as a result of the effect, so in most cases it is probably better to use a
    /// `TriggeredEffect`, with a `timed_trigger` value, in stead of using this.
    #[serde(default)]
    pub delay: f32,
}

impl TypeBody for ActiveEffectMetadata {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("kind").into(),
            ActiveEffectKind::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("sound_effect_id").into(),
            Option::<String>::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("delay").into(), f32::get_type_parts()));
    }
}

impl<'lua> FromLua<'lua> for ActiveEffectMetadata {
    fn from_lua(lua_value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
        let table = core::lua::get_table(lua_value)?;
        Ok(Self {
            kind: Box::new(table.get::<_, ActiveEffectKind>("kind")?),
            sound_effect_id: table.get("sound_effect_id")?,
            delay: table.get("delay")?,
        })
    }
}

impl<'lua> ToLua<'lua> for ActiveEffectMetadata {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("kind", *self.kind)?;
        table.set("sound_effect_id", self.sound_effect_id)?;
        table.set("delay", self.delay)?;
        lua.pack(table)
    }
}

#[derive(Clone, MluaTealDerive)]
pub struct ActiveEffectKindCircleCollider {
    radius: f32,
    passive_effects: Vec<PassiveEffectMetadata>,
    is_lethal: bool,
    is_explosion: bool,
}
impl TealData for ActiveEffectKindCircleCollider {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_active_effect_kind", |_, this, ()| {
            Ok(ActiveEffectKind::CircleCollider {
                radius: this.radius,
                passive_effects: this.passive_effects.to_owned(),
                is_lethal: this.is_lethal,
                is_explosion: this.is_explosion,
            })
        })
    }
}

#[derive(Clone, MluaTealDerive)]
pub struct ActiveEffectKindRectCollider {
    width: f32,
    height: f32,
    /// If `true` the effect will do damage to any player it hits
    is_lethal: bool,
    /// This contains any passive effects that will be spawned on collision
    passive_effects: Vec<PassiveEffectMetadata>,
}

impl TealData for ActiveEffectKindRectCollider {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_active_effect_kind", |_, this, ()| {
            Ok(ActiveEffectKind::RectCollider {
                width: this.width,
                height: this.height,
                is_lethal: this.is_lethal,
                passive_effects: this.passive_effects.to_owned(),
            })
        })
    }
}
#[derive(Clone, MluaTealDerive)]
pub struct ActiveEffectKindTriggeredEffect {
    meta: Box<TriggeredEffectMetadata>,
}
impl TealData for ActiveEffectKindTriggeredEffect {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_active_effect_kind", |_, this, ()| {
            Ok(ActiveEffectKind::TriggeredEffect {
                meta: this.meta.to_owned(),
            })
        })
    }
}

#[derive(Clone, MluaTealDerive)]
pub struct ActiveEffectKindProjectile {
    kind: ProjectileKind,
    speed: f32,
    range: f32,
    spread: f32,
    /// If `true` the effect will do damage to any player it hits
    is_lethal: bool,
    /// This contains any passive effects that will be spawned on collision
    passive_effects: Vec<PassiveEffectMetadata>,
    /// Particle effects that will be attached to the projectile
    particles: Vec<ParticleEmitterMetadata>,
}
impl TealData for ActiveEffectKindProjectile {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_active_effect_kind", |_, this, ()| {
            Ok(ActiveEffectKind::Projectile {
                kind: this.kind.to_owned(),
                speed: this.speed,
                range: this.range,
                spread: this.spread,
                is_lethal: this.is_lethal,
                passive_effects: this.passive_effects.to_owned(),
                particles: this.particles.to_owned(),
            })
        })
    }
}
/// This should hold implementations of the commonly used weapon effects, that see usage spanning
/// many different weapon implementations.
///
/// The effects that have the `Collider` suffix denote effects that do an immediate collider check,
/// upon attack, using the weapons `effect_offset` as origin.
#[derive(Clone, Serialize, Deserialize, MluaTealDerive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveEffectKind {
    /// Check for hits with a `Circle` collider.
    CircleCollider {
        radius: f32,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
        /// If `true` the effect will do damage to any player it hits
        #[serde(
            default = "core::json::default_true",
            skip_serializing_if = "core::json::is_true"
        )]
        is_lethal: bool,
        #[serde(default, skip_serializing_if = "core::json::is_false")]
        is_explosion: bool,
    },
    /// Check for hits with a `Rect` collider
    RectCollider {
        width: f32,
        height: f32,
        /// If `true` the effect will do damage to any player it hits
        #[serde(
            default = "core::json::default_true",
            skip_serializing_if = "core::json::is_true"
        )]
        is_lethal: bool,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
    },
    /// Spawn a trigger that will set of another effect if its trigger conditions are met.
    TriggeredEffect {
        #[serde(flatten)]
        meta: Box<TriggeredEffectMetadata>,
    },
    /// Spawn a projectile.
    /// This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile")]
        kind: ProjectileKind,
        speed: f32,
        range: f32,
        #[serde(default, skip_serializing_if = "f32::is_zero")]
        spread: f32,
        /// If `true` the effect will do damage to any player it hits
        #[serde(
            default = "core::json::default_true",
            skip_serializing_if = "core::json::is_true"
        )]
        is_lethal: bool,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
        /// Particle effects that will be attached to the projectile
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        particles: Vec<ParticleEmitterMetadata>,
    },
}

impl TealData for ActiveEffectKind {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("try_get_circle_collider", |_, this, ()| {
            if let ActiveEffectKind::CircleCollider {
                radius,
                passive_effects,
                is_lethal,
                is_explosion,
            } = this
            {
                Ok((
                    true,
                    Some(ActiveEffectKindCircleCollider {
                        radius: *radius,
                        passive_effects: passive_effects.to_owned(),
                        is_lethal: *is_lethal,
                        is_explosion: *is_explosion,
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_rect_collider", |_, this, ()| {
            if let ActiveEffectKind::RectCollider {
                width,
                height,
                is_lethal,
                passive_effects,
            } = this
            {
                Ok((
                    true,
                    Some(ActiveEffectKindRectCollider {
                        width: *width,
                        height: *height,
                        is_lethal: *is_lethal,
                        passive_effects: passive_effects.to_owned(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_triggered_effect", |_, this, ()| {
            if let ActiveEffectKind::TriggeredEffect { meta } = this {
                Ok((
                    true,
                    Some(ActiveEffectKindTriggeredEffect {
                        meta: meta.to_owned(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_triggered_projectile", |_, this, ()| {
            if let ActiveEffectKind::Projectile {
                kind,
                speed,
                range,
                spread,
                is_lethal,
                passive_effects,
                particles,
            } = this
            {
                Ok((
                    true,
                    Some(ActiveEffectKindProjectile {
                        kind: kind.to_owned(),
                        speed: *speed,
                        range: *range,
                        spread: *spread,
                        is_lethal: *is_lethal,
                        passive_effects: passive_effects.to_owned(),
                        particles: particles.to_owned(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        })
    }
}

pub fn debug_draw_active_effects(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let mut to_remove = Vec::new();

    let dt = get_frame_time();

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut CircleCollider)>() {
        collider.ttl_timer += dt;

        draw_circle_lines(
            transform.position.x,
            transform.position.y,
            collider.r,
            2.0,
            color::RED,
        );

        if collider.ttl_timer >= COLLIDER_DEBUG_DRAW_TTL {
            to_remove.push(e);
        }
    }

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut RectCollider)>() {
        collider.ttl_timer += dt;

        draw_rectangle_lines(
            transform.position.x,
            transform.position.y,
            collider.w,
            collider.h,
            2.0,
            color::RED,
        );

        if collider.ttl_timer >= COLLIDER_DEBUG_DRAW_TTL {
            to_remove.push(e);
        }
    }

    for e in to_remove.drain(0..) {
        world.despawn(e).unwrap();
    }
}
