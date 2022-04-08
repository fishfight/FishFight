use hv_cell::AtomicRefCell;
use hv_lua::{FromLua, ToLua};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};
use tealr::{TypeBody, TypeName};

use core::{lua::wrapped_types::RectLua, Transform};
use std::{borrow::Cow, sync::Arc};

use crate::{
    AnimatedSprite, AnimatedSpriteMetadata, AnimatedSpriteParams, CollisionWorld, Drawable,
    GameCamera, PassiveEffectInstance, PhysicsBody, Resources,
};

mod animation;
mod character;
mod controller;
mod events;
mod inventory;
mod state;

pub use animation::*;
pub use character::*;
pub use controller::*;
pub use events::*;
pub use inventory::*;
pub use state::*;

use crate::physics::PhysicsBodyParams;

pub const BODY_ANIMATED_SPRITE_ID: &str = "body";
pub const LEFT_FIN_ANIMATED_SPRITE_ID: &str = "left_fin";
pub const RIGHT_FIN_ANIMATED_SPRITE_ID: &str = "right_fin";

pub const IDLE_ANIMATION_ID: &str = "idle";
pub const MOVE_ANIMATION_ID: &str = "move";
pub const JUMP_ANIMATION_ID: &str = "jump";
pub const FALL_ANIMATION_ID: &str = "fall";
pub const CROUCH_ANIMATION_ID: &str = "crouch";
pub const SLIDE_ANIMATION_ID: &str = "slide";
pub const DEATH_BACK_ANIMATION_ID: &str = "death_back";
pub const DEATH_FORWARD_ANIMATION_ID: &str = "death_forward";

pub const WEAPON_MOUNT_TWEEN_ID: &str = "weapon_mount";
pub const ITEM_MOUNT_TWEEN_ID: &str = "item_mount";
pub const HAT_MOUNT_TWEEN_ID: &str = "hat_mount";

pub const JUMP_SOUND_ID: &str = "jump";
pub const LAND_SOUND_ID: &str = "land";

pub const RESPAWN_DELAY: f32 = 2.5;
pub const PICKUP_GRACE_TIME: f32 = 0.25;

#[derive(Debug, Clone)]
pub struct PlayerParams {
    pub index: u8,
    pub controller: PlayerControllerKind,
    pub character: PlayerCharacterMetadata,
}

#[derive(Clone, TypeName)]
pub struct Player {
    pub index: u8,
    pub state: PlayerState,
    pub damage_from_left: bool,
    pub is_facing_left: bool,
    pub is_upside_down: bool,
    pub is_attacking: bool,
    pub jump_frame_counter: u16,
    pub pickup_grace_timer: f32,
    pub incapacitation_timer: f32,
    pub attack_timer: f32,
    pub respawn_timer: f32,
    pub camera_box: Rect,
    pub passive_effects: Vec<PassiveEffectInstance>,
}

impl<'lua> FromLua<'lua> for Player {
    fn from_lua(lua_value: hv_lua::Value<'lua>, _: &'lua hv_lua::Lua) -> hv_lua::Result<Self> {
        let table = core::lua::get_table(lua_value)?;
        Ok(Self {
            index: table.get("index")?,
            state: table.get("state")?,
            damage_from_left: table.get("damage_from_left")?,
            is_facing_left: table.get("is_facing_left")?,
            is_upside_down: table.get("is_upside_down")?,
            is_attacking: table.get("is_attacking")?,
            jump_frame_counter: table.get("jump_frame_counter")?,
            pickup_grace_timer: table.get("pickup_grace_timer")?,
            incapacitation_timer: table.get("incapacitation_timer")?,
            attack_timer: table.get("attack_timer")?,
            respawn_timer: table.get("respawn_timer")?,
            camera_box: table.get::<_, RectLua>("camera_box")?.into(),
            passive_effects: table.get("passive_effects")?,
        })
    }
}

impl<'lua> ToLua<'lua> for Player {
    fn to_lua(self, lua: &'lua hv_lua::Lua) -> hv_lua::Result<hv_lua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("index", self.index)?;
        table.set("state", self.state)?;
        table.set("damage_from_left", self.damage_from_left)?;
        table.set("is_facing_left", self.is_facing_left)?;
        table.set("is_upside_down", self.is_upside_down)?;
        table.set("is_attacking", self.is_attacking)?;
        table.set("jump_frame_counter", self.jump_frame_counter)?;
        table.set("pickup_grace_timer", self.pickup_grace_timer)?;
        table.set("incapacitation_timer", self.incapacitation_timer)?;
        table.set("attack_timer", self.attack_timer)?;
        table.set("respawn_timer", self.respawn_timer)?;
        table.set("camera_box", RectLua::from(self.camera_box))?;
        table.set("passive_effects", self.passive_effects)?;
        lua.pack(table)
    }
}

impl TypeBody for Player {
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.fields
            .push((Cow::Borrowed("index").into(), u8::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("state").into(), PlayerState::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("damage_from_left").into(),
            bool::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("is_facing_left").into(),
            bool::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("is_upside_down").into(),
            bool::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("is_attacking").into(), bool::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("jump_frame_counter").into(),
            u16::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("pickup_grace_timer").into(),
            f32::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("incapacitation_timer").into(),
            f32::get_type_parts(),
        ));
        gen.fields
            .push((Cow::Borrowed("attack_timer").into(), f32::get_type_parts()));
        gen.fields
            .push((Cow::Borrowed("respawn_timer").into(), f32::get_type_parts()));
        gen.fields.push((
            Cow::Borrowed("camera_box").into(),
            RectLua::get_type_parts(),
        ));
        gen.fields.push((
            Cow::Borrowed("passive_effects").into(),
            Vec::<PassiveEffectInstance>::get_type_parts(),
        ));
    }
}

impl Player {
    pub fn new(index: u8, position: Vec2) -> Self {
        let camera_box = Rect::new(position.x - 30.0, position.y - 150.0, 100.0, 210.0);

        Player {
            index,
            state: PlayerState::None,
            damage_from_left: false,
            is_facing_left: false,
            is_upside_down: false,
            is_attacking: false,
            jump_frame_counter: 0,
            pickup_grace_timer: 0.0,
            attack_timer: 0.0,
            incapacitation_timer: 0.0,
            respawn_timer: 0.0,
            camera_box,
            passive_effects: Vec::new(),
        }
    }
}

pub fn update_player_camera_box(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, (transform, player)) in world.query_mut::<(&Transform, &mut Player)>() {
        let rect = Rect::new(transform.position.x, transform.position.y, 32.0, 60.0);

        if rect.x < player.camera_box.x {
            player.camera_box.x = rect.x;
        }

        if rect.x + rect.w > player.camera_box.x + player.camera_box.w {
            player.camera_box.x = rect.x + rect.w - player.camera_box.w;
        }

        if rect.y < player.camera_box.y {
            player.camera_box.y = rect.y;
        }

        if rect.y + rect.h > player.camera_box.y + player.camera_box.h {
            player.camera_box.y = rect.y + rect.h - player.camera_box.h;
        }

        let mut camera = storage::get_mut::<GameCamera>();
        camera.add_player_rect(player.camera_box);
    }
}

#[derive(Debug, Clone)]
pub struct PlayerAttributes {
    pub head_threshold: f32,
    pub legs_threshold: f32,
    pub weapon_mount: Vec2,
    pub jump_force: f32,
    pub move_speed: f32,
    pub slide_speed_factor: f32,
    pub incapacitation_duration: f32,
    pub float_gravity_factor: f32,
}

impl From<&PlayerCharacterMetadata> for PlayerAttributes {
    fn from(params: &PlayerCharacterMetadata) -> Self {
        PlayerAttributes {
            head_threshold: params.head_threshold,
            legs_threshold: params.legs_threshold,
            weapon_mount: params.weapon_mount,
            jump_force: params.jump_force,
            move_speed: params.move_speed,
            slide_speed_factor: params.slide_speed_factor,
            incapacitation_duration: params.incapacitation_duration,
            float_gravity_factor: params.float_gravity_factor,
        }
    }
}

impl From<PlayerCharacterMetadata> for PlayerAttributes {
    fn from(params: PlayerCharacterMetadata) -> Self {
        PlayerAttributes::from(&params)
    }
}

pub fn spawn_player(
    world: &mut World,
    index: u8,
    position: Vec2,
    controller: PlayerControllerKind,
    character: PlayerCharacterMetadata,
) -> Entity {
    let weapon_mount = character.weapon_mount;
    let item_mount = character.item_mount;
    let hat_mount = character.hat_mount;

    let offset = storage::get::<Resources>()
        .textures
        .get(&character.sprite.texture_id)
        .map(|t| {
            let frame_size = t.frame_size();
            character.sprite.offset
                - vec2(frame_size.x / 2.0, frame_size.y - character.collider_size.y)
        })
        .unwrap();

    let animations = character
        .sprite
        .animations
        .to_vec()
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<_>>();

    let texture_id = character.sprite.texture_id.clone();

    let params = {
        let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

        AnimatedSpriteParams {
            offset,
            ..meta.into()
        }
    };

    let sprites = vec![(
        BODY_ANIMATED_SPRITE_ID,
        AnimatedSprite::new(&texture_id, animations.as_slice(), params),
    )];

    let draw_order = (index as u32 + 1) * 10;

    let size = character.collider_size.as_i32();
    let actor = storage::get_mut::<CollisionWorld>().add_actor(position, size.x, size.y);

    let body_params = PhysicsBodyParams {
        offset: vec2(-character.collider_size.x / 2.0, 0.0),
        size: character.collider_size,
        has_friction: false,
        can_rotate: false,
        gravity: character.gravity,
        ..Default::default()
    };

    world.spawn((
        Player::new(index, position),
        Transform::from(position),
        PlayerController::from(controller),
        PlayerAttributes::from(&character),
        PlayerInventory::new(weapon_mount, item_mount, hat_mount),
        PlayerEventQueue::new(),
        Drawable::new_animated_sprite_set(draw_order, &sprites),
        PhysicsBody::new(actor, None, body_params),
    ))
}
