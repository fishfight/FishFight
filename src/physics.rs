use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, ToLua};
use macroquad::color;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use macroquad_platformer::{Actor, Tile};

use serde::{Deserialize, Serialize};

use hecs::World;
use tealr::{TypeBody, TypeName};

use crate::{
    lua::{run_event, ActorLua},
    CollisionWorld, Map,
};
use core::{
    lua::{get_table, wrapped_types::Vec2Lua},
    Transform,
};
use std::{borrow::Cow, sync::Arc};

pub const GRAVITY: f32 = 2.5;
pub const TERMINAL_VELOCITY: f32 = 10.0;

pub fn create_collision_world(map: &Map) -> CollisionWorld {
    let tile_cnt = (map.grid_size.x * map.grid_size.y) as usize;
    let mut static_colliders = Vec::with_capacity(tile_cnt);
    for _ in 0..tile_cnt {
        static_colliders.push(Tile::Empty);
    }

    for layer_id in &map.draw_order {
        let layer = map.layers.get(layer_id).unwrap();
        if layer.has_collision {
            for (i, (_, _, tile)) in map.get_tiles(layer_id, None).enumerate() {
                if let Some(tile) = tile {
                    if tile
                        .attributes
                        .contains(&Map::PLATFORM_TILE_ATTRIBUTE.to_string())
                    {
                        static_colliders[i] = Tile::JumpThrough;
                    } else {
                        static_colliders[i] = Tile::Solid;
                    }
                }
            }
        }
    }

    let mut collision_world = CollisionWorld::new();
    collision_world.add_static_tiled_layer(
        static_colliders,
        map.tile_size.x,
        map.tile_size.y,
        map.grid_size.x as usize,
        1,
    );

    collision_world
}

const FRICTION_LERP: f32 = 0.96;
const STOP_THRESHOLD: f32 = 1.0;

#[derive(Debug, Clone, TypeName)]
pub struct PhysicsBodyParams {
    pub size: Vec2,
    pub offset: Vec2,
    pub has_mass: bool,
    pub has_friction: bool,
    pub can_rotate: bool,
    pub bouncyness: f32,
    pub gravity: f32,
}

impl Default for PhysicsBodyParams {
    fn default() -> Self {
        PhysicsBodyParams {
            size: vec2(16.0, 16.0),
            offset: Vec2::ZERO,
            has_mass: true,
            has_friction: true,
            can_rotate: true,
            bouncyness: 0.0,
            gravity: GRAVITY,
        }
    }
}

impl<'lua> FromLua<'lua> for PhysicsBodyParams {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = match lua_value {
            hv_lua::Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "table",
                    message: None,
                })
            }
        };
        Ok(Self {
            size: table.get::<_, Vec2Lua>("size")?.into(),
            offset: table.get::<_, Vec2Lua>("offset")?.into(),
            has_mass: table.get("has_mass")?,
            has_friction: table.get("has_friction")?,
            can_rotate: table.get("can_rotate")?,
            bouncyness: table.get("bouncyness")?,
            gravity: table.get("gravity")?,
        })
    }
}
impl<'lua> ToLua<'lua> for PhysicsBodyParams {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("size", Vec2Lua::from(self.size))?;
        table.set("offset", Vec2Lua::from(self.offset))?;
        table.set("has_mass", self.has_mass)?;
        table.set("has_friction", self.has_friction)?;
        table.set("can_rotate", self.can_rotate)?;
        table.set("bouncyness", self.bouncyness)?;
        table.set("gravity", self.gravity)?;
        lua.pack(table)
    }
}
impl TypeBody for PhysicsBodyParams {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("size").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("has_mass").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("has_friction").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("can_rotate").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("bouncyness").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("gravity").into(), f32::get_type_parts()));
    }
}

/// Regular simulated physics bodies.
/// Note that rotation is abstract, only set on the transform to be used for draws. The colliders
/// are axis-aligned and will not be affected by rotation.
#[derive(Clone, tealr::TypeName)]
pub struct PhysicsBody {
    pub actor: Actor,
    pub offset: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub is_on_ground: bool,
    pub was_on_ground: bool,
    /// Will be `true` if the body is currently on top of a platform/jumpthrough tile
    pub is_on_platform: bool,
    /// If this is `true` the body will be affected by gravity
    pub has_mass: bool,
    pub has_friction: bool,
    pub can_rotate: bool,
    pub bouncyness: f32,
    pub is_deactivated: bool,
    pub gravity: f32,
}
impl<'lua> FromLua<'lua> for PhysicsBody {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = match lua_value {
            hv_lua::Value::Table(x) => x,
            x => {
                return Err(hv_lua::Error::FromLuaConversionError {
                    from: x.type_name(),
                    to: "table",
                    message: None,
                })
            }
        };
        Ok(Self {
            actor: table.get::<_, ActorLua>("actor")?.into(),
            offset: table.get::<_, Vec2Lua>("offset")?.into(),
            size: table.get::<_, Vec2Lua>("size")?.into(),
            velocity: table.get::<_, Vec2Lua>("velocity")?.into(),
            is_on_ground: table.get("is_on_ground")?,
            was_on_ground: table.get("was_on_ground")?,
            is_on_platform: table.get("is_on_platform")?,
            has_mass: table.get("has_mass")?,
            has_friction: table.get("has_friction")?,
            can_rotate: table.get("can_rotate")?,
            bouncyness: table.get("bouncyness")?,
            is_deactivated: table.get("is_deactivated")?,
            gravity: table.get("gravity")?,
        })
    }
}
impl TypeBody for PhysicsBody {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("actor").into(), ActorLua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("size").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("velocity").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("is_on_ground").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("was_on_ground").into(),
            bool::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("is_on_platform").into(),
            bool::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("has_mass").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("has_friction").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("can_rotate").into(), bool::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("bouncyness").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("is_deactivated").into(),
            bool::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("gravity").into(), f32::get_type_parts()));
    }
}
impl<'lua> ToLua<'lua> for PhysicsBody {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("actor", ActorLua::from(self.actor))?;
        table.set("offset", Vec2Lua::from(self.offset))?;
        table.set("size", Vec2Lua::from(self.size))?;
        table.set("velocity", Vec2Lua::from(self.velocity))?;
        table.set("is_on_ground", self.is_on_ground)?;
        table.set("was_on_ground", self.was_on_ground)?;
        table.set("is_on_platform", self.is_on_platform)?;
        table.set("has_mass", self.has_mass)?;
        table.set("has_friction", self.has_friction)?;
        table.set("can_rotate", self.can_rotate)?;
        table.set("bouncyness", self.bouncyness)?;
        table.set("is_deactivated", self.is_deactivated)?;
        table.set("gravity", self.gravity)?;
        lua.pack(table)
    }
}

impl PhysicsBody {
    pub fn new<V: Into<Option<Vec2>>>(
        actor: Actor,
        velocity: V,
        params: PhysicsBodyParams,
    ) -> Self {
        let velocity = velocity.into().unwrap_or_default();

        PhysicsBody {
            actor,
            offset: params.offset,
            size: params.size,
            velocity,
            is_on_ground: false,
            was_on_ground: false,
            is_on_platform: false,
            has_mass: params.has_mass,
            has_friction: params.has_friction,
            can_rotate: params.can_rotate,
            bouncyness: params.bouncyness,
            is_deactivated: false,
            gravity: params.gravity,
        }
    }

    pub fn as_rect(&self, position: Vec2) -> Rect {
        let position = position + self.offset;
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }
}

pub fn fixed_update_physics_bodies(world: Arc<AtomicRefCell<World>>) {
    {
        let mut world = AtomicRefCell::borrow_mut(world.as_ref());
        let mut collision_world = storage::get_mut::<CollisionWorld>();

        for (_, (transform, body)) in world.query_mut::<(&mut Transform, &mut PhysicsBody)>() {
            collision_world.set_actor_position(body.actor, transform.position + body.offset);

            if !body.is_deactivated {
                let position = collision_world.actor_pos(body.actor);

                {
                    let position = position + vec2(0.0, 1.0);

                    body.was_on_ground = body.is_on_ground;

                    body.is_on_ground = collision_world.collide_check(body.actor, position);

                    // FIXME: Using this to set `is_on_ground` caused weird glitching behavior when jumping up through platforms
                    let tile = collision_world.collide_solids(
                        position,
                        body.size.x as i32,
                        body.size.y as i32,
                    );

                    body.is_on_platform = tile == Tile::JumpThrough;
                }

                if !body.is_on_ground && body.has_mass {
                    body.velocity.y += body.gravity;

                    if body.velocity.y > TERMINAL_VELOCITY {
                        body.velocity.y = TERMINAL_VELOCITY;
                    }
                }

                if !collision_world.move_h(body.actor, body.velocity.x) {
                    body.velocity.x *= -body.bouncyness;
                }

                if !collision_world.move_v(body.actor, body.velocity.y) {
                    body.velocity.y *= -body.bouncyness;
                }

                if body.can_rotate {
                    apply_rotation(transform, &mut body.velocity, body.is_on_ground);
                }

                if body.is_on_ground && body.has_friction {
                    body.velocity.x *= FRICTION_LERP;
                    if body.velocity.x.abs() <= STOP_THRESHOLD {
                        body.velocity.x = 0.0;
                    }
                }

                transform.position = collision_world.actor_pos(body.actor) - body.offset;
            }
        }
    }
    let _ = run_event("fixed_update_physics_bodies", world)
        .map_err(|v| eprintln!("Ran into an error:\n{}", v));
}

pub fn debug_draw_physics_bodies(world: Arc<AtomicRefCell<World>>) {
    let world = AtomicRefCell::borrow(world.as_ref());
    for (_, (transform, body)) in world.query::<(&Transform, &PhysicsBody)>().iter() {
        if !body.is_deactivated {
            let rect = body.as_rect(transform.position);

            let color = if body.is_on_platform {
                color::YELLOW
            } else if body.is_on_ground {
                color::RED
            } else {
                color::GREEN
            };

            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, color);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeName)]
pub struct RigidBodyParams {
    #[serde(with = "core::json::vec2_def")]
    pub offset: Vec2,
    #[serde(with = "core::json::vec2_def")]
    pub size: Vec2,
    #[serde(default, skip_serializing_if = "core::json::is_false")]
    pub can_rotate: bool,
}

impl Default for RigidBodyParams {
    fn default() -> Self {
        RigidBodyParams {
            offset: Vec2::ZERO,
            size: vec2(16.0, 16.0),
            can_rotate: false,
        }
    }
}

impl TypeBody for RigidBodyParams {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("size").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("can_rotate").into(), bool::get_type_parts()));
    }
}

/// Simple physics bodies that has a velocity and optional rotation.
/// Note that rotation is abstract, only set on the transform to be used for draws. The colliders
/// are axis-aligned and will not be affected by rotation.
#[derive(Clone, TypeName)]
pub struct RigidBody {
    pub offset: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub can_rotate: bool,
}

impl<'lua> FromLua<'lua> for RigidBody {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = get_table(lua_value)?;
        Ok(Self {
            offset: table.get::<_, Vec2Lua>("offset")?.into(),
            size: table.get::<_, Vec2Lua>("size")?.into(),
            velocity: table.get::<_, Vec2Lua>("velocity")?.into(),
            can_rotate: table.get("can_rotate")?,
        })
    }
}
impl<'lua> ToLua<'lua> for RigidBody {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("offset", Vec2Lua::from(self.offset))?;
        table.set("size", Vec2Lua::from(self.size))?;
        table.set("velocity", Vec2Lua::from(self.velocity))?;
        table.set("can_rotate", self.can_rotate)?;
        lua.pack(table)
    }
}

impl TypeBody for RigidBody {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("offset").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("size").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("velocity").into(), Vec2Lua::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("can_rotate").into(), bool::get_type_parts()));
    }
}

impl RigidBody {
    pub fn new<V: Into<Option<Vec2>>>(velocity: V, params: RigidBodyParams) -> Self {
        let velocity = velocity.into().unwrap_or_default();

        RigidBody {
            offset: params.offset,
            size: params.size,
            velocity,
            can_rotate: params.can_rotate,
        }
    }

    pub fn as_rect(&self, position: Vec2) -> Rect {
        let position = position + self.offset;
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }
}

pub fn fixed_update_rigid_bodies(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, (transform, body)) in world.query_mut::<(&mut Transform, &mut RigidBody)>() {
        transform.position += body.velocity;

        if body.can_rotate {
            apply_rotation(transform, &mut body.velocity, false);
        }
    }
}

pub fn debug_draw_rigid_bodies(world: Arc<AtomicRefCell<World>>) {
    let world = AtomicRefCell::borrow(world.as_ref());
    for (_, (transform, body)) in world.query::<(&Transform, &RigidBody)>().iter() {
        let rect = body.as_rect(transform.position);

        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, color::RED)
    }
}

fn apply_rotation(transform: &mut Transform, velocity: &mut Vec2, is_on_ground: bool) {
    if !is_on_ground {
        transform.rotation += velocity.x.abs() * 0.00045 + velocity.y.abs() * 0.00015;
    } else {
        transform.rotation %= std::f32::consts::PI * 2.0;

        let goal = std::f32::consts::PI * 2.0;

        let rest = goal - transform.rotation;
        if rest.abs() >= 0.1 {
            transform.rotation += (rest * 0.1).max(0.1);
        }
    }
}
