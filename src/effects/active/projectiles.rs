use core::lua::get_table;
use core::lua::wrapped_types::{ColorLua, Vec2Lua};
use hv_cell::AtomicRefCell;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use mlua::{FromLua, ToLua};
use std::borrow::Cow;
use std::f32::consts::PI;
use std::sync::Arc;
use tealr::mlu::TealData;
use tealr::{MluaTealDerive, TypeBody, TypeName};

use hecs::{Entity, World};
use macroquad_platformer::Tile;

use serde::{Deserialize, Serialize};

use crate::effects::active::triggered::TriggeredEffect;
use crate::effects::TriggeredEffectTrigger;
use crate::particles::{ParticleEmitter, ParticleEmitterMetadata};
use crate::player::{on_player_damage, Player, PlayerState};
use crate::{CollisionWorld, PhysicsBody, Resources, RigidBody, RigidBodyParams, SpriteMetadata};
use crate::{Drawable, PassiveEffectInstance, PassiveEffectMetadata, SpriteParams};
use core::Transform;

const PROJECTILE_DRAW_ORDER: u32 = 1;

use hv_lua as mlua;

#[derive(Clone, MluaTealDerive)]
pub struct Circle {
    radius: f32,
    color: ColorLua,
}
impl TealData for Circle {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_projectile_kind", |_, this, ()| {
            Ok(ProjectileKind::Circle {
                radius: this.radius,
                color: this.color.to_owned().into(),
            })
        })
    }
}

#[derive(Clone, MluaTealDerive)]
pub struct Rectangle {
    width: f32,
    height: f32,
    color: ColorLua,
}
impl TealData for Rectangle {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_projectile_kind", |_, this, ()| {
            Ok(ProjectileKind::Rect {
                width: this.width,
                height: this.height,
                color: this.color.to_owned().into(),
            })
        })
    }
}
#[derive(Clone, MluaTealDerive)]
pub struct SpriteProjectile {
    params: SpriteMetadata,
    can_rotate: bool,
}
impl TealData for SpriteProjectile {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("to_projectile_kind", |_, this, ()| {
            Ok(ProjectileKind::Sprite {
                params: this.params.to_owned(),
                can_rotate: this.can_rotate,
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, MluaTealDerive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectileKind {
    Circle {
        radius: f32,
        #[serde(with = "core::json::ColorDef")]
        color: Color,
    },
    Rect {
        width: f32,
        height: f32,
        #[serde(with = "core::json::ColorDef")]
        color: Color,
    },
    Sprite {
        #[serde(rename = "sprite")]
        params: SpriteMetadata,
        /// If yes, the sprite would be rotated by angle between Vec2(1, 0) (most likely will be changed in the future) and velocity vector.
        /// This, for example, used for machine gun bullets rotation.
        #[serde(default)]
        can_rotate: bool,
    },
}
impl TealData for ProjectileKind {
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("try_get_circle", |_, this, ()| {
            if let ProjectileKind::Circle { radius, color } = this {
                Ok((
                    true,
                    Some(Circle {
                        radius: radius.to_owned(),
                        color: color.to_owned().into(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_rect", |_, this, ()| {
            if let ProjectileKind::Rect {
                width,
                height,
                color,
            } = this
            {
                Ok((
                    true,
                    Some(Rectangle {
                        width: *width,
                        height: *height,
                        color: color.to_owned().into(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
        methods.add_method("try_get_sprite", |_, this, ()| {
            if let ProjectileKind::Sprite { can_rotate, params } = this {
                Ok((
                    true,
                    Some(SpriteProjectile {
                        can_rotate: *can_rotate,
                        params: params.to_owned(),
                    }),
                ))
            } else {
                Ok((false, None))
            }
        });
    }
}

#[derive(Clone, TypeName)]
pub struct Projectile {
    pub kind: ProjectileKind,
    pub owner: Entity,
    pub origin: Vec2,
    pub range: f32,
    pub is_lethal: bool,
    pub passive_effects: Vec<PassiveEffectMetadata>,
}

impl<'lua> FromLua<'lua> for Projectile {
    fn from_lua(lua_value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            kind: table.get("kind")?,
            owner: table.get("owner")?,
            origin: table.get::<_, Vec2Lua>("origin")?.into(),
            range: table.get("range")?,
            is_lethal: table.get("is_lethal")?,
            passive_effects: table.get("passive_effects")?,
        })
    }
}
impl<'lua> ToLua<'lua> for Projectile {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("kind", self.kind)?;
        table.set("owner", self.owner)?;
        table.set("origin", Vec2Lua::from(self.origin))?;
        table.set("range", self.range)?;
        table.set("is_lethal", self.is_lethal)?;
        table.set("passive_effects", self.passive_effects)?;
        lua.pack(table)
    }
}

impl TypeBody for Projectile {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields.push((
            Cow::Borrowed("kind").into(),
            ProjectileKind::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("owner").into(), Entity::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("origin").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("range").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("is_lethal").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("passive_effects").into(),
            Vec::<PassiveEffectMetadata>::get_type_parts(),
        ));
    }
}

impl Projectile {
    pub fn new(
        owner: Entity,
        kind: ProjectileKind,
        origin: Vec2,
        range: f32,
        is_lethal: bool,
        passive_effects: &[PassiveEffectMetadata],
    ) -> Self {
        Projectile {
            owner,
            kind,
            origin,
            range,
            is_lethal,
            passive_effects: passive_effects.to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct ProjectileParams {
    pub is_lethal: bool,
    pub passive_effects: Vec<PassiveEffectMetadata>,
    pub particle_effects: Vec<ParticleEmitterMetadata>,
}

impl Default for ProjectileParams {
    fn default() -> Self {
        ProjectileParams {
            is_lethal: true,
            passive_effects: Vec::new(),
            particle_effects: Vec::new(),
        }
    }
}

pub fn spawn_projectile(
    world: &mut World,
    owner: Entity,
    kind: ProjectileKind,
    origin: Vec2,
    velocity: Vec2,
    range: f32,
    params: ProjectileParams,
) -> Entity {
    let entity = world.spawn(());
    world
        .insert_one(
            entity,
            Projectile::new(
                owner,
                kind.clone(),
                origin,
                range,
                params.is_lethal,
                &params.passive_effects,
            ),
        )
        .unwrap();

    let mut transform = Transform::from(origin);

    let body_params = match kind {
        ProjectileKind::Rect { width, height, .. } => RigidBodyParams {
            offset: vec2(-width, -height) / 2.0,
            size: vec2(width, height),
            can_rotate: false,
        },
        ProjectileKind::Circle { radius, .. } => RigidBodyParams {
            size: vec2(radius * 2.0, radius * 2.0),
            can_rotate: false,
            ..Default::default()
        },
        ProjectileKind::Sprite {
            params: meta,
            can_rotate,
        } => {
            let resources = storage::get::<Resources>();
            let texture_res = resources.textures.get(&meta.texture_id).unwrap();

            let size = meta
                .size
                .unwrap_or_else(|| texture_res.meta.frame_size.unwrap_or(texture_res.meta.size));

            let offset = meta.offset - (vec2(size.x, size.y) / 2.0);

            let is_flipped_x = velocity.x < 0.0;

            if can_rotate {
                let mut direction = Vec2::ZERO;

                if is_flipped_x {
                    direction.x = 1.0;
                } else {
                    direction.x = -1.0;
                }

                transform.rotation = (velocity.y - direction.y).atan2(velocity.x - direction.x);

                if is_flipped_x {
                    transform.rotation += PI;
                }
            }

            world
                .insert_one(
                    entity,
                    Drawable::new_sprite(
                        PROJECTILE_DRAW_ORDER,
                        &meta.texture_id,
                        SpriteParams {
                            is_flipped_x,
                            offset,
                            ..meta.clone().into()
                        },
                    ),
                )
                .unwrap();

            RigidBodyParams {
                offset,
                size,
                ..Default::default()
            }
        }
    };

    world
        .insert(entity, (transform, RigidBody::new(velocity, body_params)))
        .unwrap();

    let mut particle_emitters = Vec::new();
    for params in params.particle_effects {
        let mut emitter = ParticleEmitter::from(params);
        emitter.is_active = true;

        particle_emitters.push(emitter);
    }

    if !particle_emitters.is_empty() {
        world.insert_one(entity, particle_emitters).unwrap();
    }

    entity
}

enum ProjectileCollision {
    Player(Entity),
    Trigger(Entity),
    Map,
}

pub fn fixed_update_projectiles(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    let bodies = world
        .query::<(&Transform, &PhysicsBody)>()
        .iter()
        .map(|(e, (transform, body))| (e, body.as_rect(transform.position)))
        .collect::<Vec<_>>();

    let collision_world = storage::get::<CollisionWorld>();

    let mut events = Vec::new();

    'projectiles: for (e, (projectile, transform, body)) in world
        .query::<(&Projectile, &Transform, &RigidBody)>()
        .iter()
    {
        if projectile.origin.distance(transform.position) >= projectile.range {
            events.push((projectile.owner, e, None));
            continue 'projectiles;
        }

        let size = body.size.as_i32();
        let map_collision = collision_world.collide_solids(transform.position, size.x, size.y);
        if map_collision == Tile::Solid {
            let res = (projectile.owner, e, Some(ProjectileCollision::Map));
            events.push(res);
            continue 'projectiles;
        }

        let rect = body.as_rect(transform.position);
        for (other, other_rect) in &bodies {
            if rect.overlaps(other_rect) {
                if let Ok(mut player) = world.get_mut::<Player>(*other) {
                    if player.state != PlayerState::Dead {
                        for meta in projectile.passive_effects.clone().into_iter() {
                            let effect_instance = PassiveEffectInstance::new(None, meta);

                            player.passive_effects.push(effect_instance);
                        }

                        if projectile.is_lethal {
                            let res = (
                                projectile.owner,
                                e,
                                Some(ProjectileCollision::Player(*other)),
                            );

                            events.push(res);
                        }

                        continue 'projectiles;
                    }
                } else if let Ok(effect) = world.get::<TriggeredEffect>(*other) {
                    if effect.trigger.contains(&TriggeredEffectTrigger::Projectile) {
                        let res = (
                            projectile.owner,
                            e,
                            Some(ProjectileCollision::Trigger(*other)),
                        );
                        events.push(res);
                    }
                }
            }
        }
    }

    for (damage_from_entity, projectile_entity, collision) in events {
        if let Some(collision_kind) = collision {
            match collision_kind {
                ProjectileCollision::Player(damage_to_entity) => {
                    on_player_damage(&mut world, damage_from_entity, damage_to_entity);
                }
                ProjectileCollision::Trigger(trigger_entity) => {
                    let mut effect = world.get_mut::<TriggeredEffect>(trigger_entity).unwrap();
                    if !effect.should_override_delay {
                        effect.is_triggered = true;
                        effect.should_override_delay = true;
                        effect.triggered_by = Some(damage_from_entity);
                    }
                }
                _ => {}
            }
        }

        let _ = world.despawn(projectile_entity);
    }
}

pub fn draw_projectiles(world: &mut World) {
    for (_, (projectile, transform)) in world.query::<(&Projectile, &Transform)>().iter() {
        match projectile.kind {
            ProjectileKind::Rect {
                width,
                height,
                color,
            } => {
                draw_rectangle(
                    transform.position.x,
                    transform.position.y,
                    width,
                    height,
                    color,
                );
            }
            ProjectileKind::Circle { radius, color } => {
                draw_circle(transform.position.x, transform.position.y, radius, color);
            }
            _ => {}
        }
    }
}
