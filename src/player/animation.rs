use std::sync::Arc;

use hv_cell::AtomicRefCell;
use macroquad::prelude::*;

use hecs::World;

use serde::{Deserialize, Serialize};

use crate::player::{
    Player, PlayerInventory, PlayerState, BODY_ANIMATED_SPRITE_ID, CROUCH_ANIMATION_ID,
    DEATH_BACK_ANIMATION_ID, DEATH_FORWARD_ANIMATION_ID, FALL_ANIMATION_ID, HAT_MOUNT_TWEEN_ID,
    IDLE_ANIMATION_ID, ITEM_MOUNT_TWEEN_ID, JUMP_ANIMATION_ID, MOVE_ANIMATION_ID,
    SLIDE_ANIMATION_ID, WEAPON_MOUNT_TWEEN_ID,
};
use crate::{AnimatedSpriteMetadata, AnimationMetadata, Keyframe, TweenMetadata};
use crate::{Drawable, PhysicsBody};

/// This is used in stead of `AnimationParams`, as we have different data requirements, in the case
/// of a player character, compared to most other use cases. We want to have a default animation
/// set, for instance, that corresponds with the way the core game characters are animated, but
/// still have the possibility to declare custom animation sets, as well as have variation in size,
///
/// Refer to `crate::components::animation_player::AnimationParams` for detailed documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimationMetadata {
    #[serde(rename = "texture")]
    pub texture_id: String,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default, with = "core::json::vec2_def")]
    pub offset: Vec2,
    #[serde(default, with = "core::json::vec2_opt")]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "core::json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    #[serde(default)]
    pub animations: PlayerAnimations,
}

impl From<PlayerAnimationMetadata> for AnimatedSpriteMetadata {
    fn from(other: PlayerAnimationMetadata) -> Self {
        AnimatedSpriteMetadata {
            texture_id: other.texture_id,
            scale: other.scale,
            offset: other.offset,
            pivot: other.pivot,
            tint: other.tint,
            animations: other.animations.into_vec(),
            autoplay_id: Some(IDLE_ANIMATION_ID.to_string()),
            is_deactivated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimations {
    #[serde(default = "PlayerAnimations::default_idle_animation")]
    pub idle: AnimationMetadata,
    #[serde(
        rename = "move",
        alias = "moving",
        default = "PlayerAnimations::default_move_animation"
    )]
    pub moving: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_jump_animation")]
    pub jump: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_fall_animation")]
    pub fall: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_crouch_animation")]
    pub crouch: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_slide_animation")]
    pub slide: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_death_back_animation")]
    pub death_back: AnimationMetadata,
    #[serde(default = "PlayerAnimations::default_death_forward_animation")]
    pub death_forward: AnimationMetadata,
}

impl PlayerAnimations {
    pub fn default_idle_animation() -> AnimationMetadata {
        let keyframes = vec![
            Keyframe {
                frame: 0,
                translation: vec2(0.0, 0.0),
            },
            Keyframe {
                frame: 5,
                translation: vec2(0.0, 2.0),
            },
            Keyframe {
                frame: 8,
                translation: vec2(0.0, 0.0),
            },
            Keyframe {
                frame: 11,
                translation: vec2(0.0, 2.0),
            },
        ];

        AnimationMetadata {
            id: IDLE_ANIMATION_ID.to_string(),
            row: 0,
            frames: 14,
            fps: 12,
            tweens: vec![
                TweenMetadata {
                    id: WEAPON_MOUNT_TWEEN_ID.to_string(),
                    keyframes: keyframes.clone(),
                },
                TweenMetadata {
                    id: ITEM_MOUNT_TWEEN_ID.to_string(),
                    keyframes: keyframes.clone(),
                },
                TweenMetadata {
                    id: HAT_MOUNT_TWEEN_ID.to_string(),
                    keyframes,
                },
            ],
            is_looping: true,
        }
    }

    pub fn default_move_animation() -> AnimationMetadata {
        let keyframes = vec![
            Keyframe {
                frame: 0,
                translation: vec2(0.0, -4.0),
            },
            Keyframe {
                frame: 2,
                translation: vec2(0.0, 0.0),
            },
            Keyframe {
                frame: 5,
                translation: vec2(0.0, -4.0),
            },
        ];

        AnimationMetadata {
            id: MOVE_ANIMATION_ID.to_string(),
            row: 1,
            frames: 6,
            fps: 10,
            tweens: vec![
                TweenMetadata {
                    id: WEAPON_MOUNT_TWEEN_ID.to_string(),
                    keyframes: keyframes.clone(),
                },
                TweenMetadata {
                    id: ITEM_MOUNT_TWEEN_ID.to_string(),
                    keyframes,
                },
                TweenMetadata {
                    id: HAT_MOUNT_TWEEN_ID.to_string(),
                    keyframes: vec![
                        Keyframe {
                            frame: 0,
                            translation: vec2(4.0, -4.0),
                        },
                        Keyframe {
                            frame: 2,
                            translation: vec2(4.0, 0.0),
                        },
                        Keyframe {
                            frame: 5,
                            translation: vec2(4.0, -4.0),
                        },
                    ],
                },
            ],
            is_looping: true,
        }
    }

    pub fn default_jump_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: JUMP_ANIMATION_ID.to_string(),
            row: 2,
            frames: 1,
            fps: 5,
            tweens: Vec::new(),
            is_looping: false,
        }
    }

    pub fn default_fall_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: FALL_ANIMATION_ID.to_string(),
            row: 3,
            frames: 1,
            fps: 8,
            tweens: Vec::new(),
            is_looping: true,
        }
    }

    pub fn default_crouch_animation() -> AnimationMetadata {
        let keyframes = vec![Keyframe {
            frame: 0,
            translation: vec2(0.0, 4.0),
        }];

        AnimationMetadata {
            id: CROUCH_ANIMATION_ID.to_string(),
            row: 4,
            frames: 1,
            fps: 8,
            tweens: vec![
                TweenMetadata {
                    id: WEAPON_MOUNT_TWEEN_ID.to_string(),
                    keyframes: keyframes.clone(),
                },
                TweenMetadata {
                    id: ITEM_MOUNT_TWEEN_ID.to_string(),
                    keyframes: keyframes.clone(),
                },
                TweenMetadata {
                    id: HAT_MOUNT_TWEEN_ID.to_string(),
                    keyframes,
                },
            ],
            is_looping: false,
        }
    }

    pub fn default_slide_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: SLIDE_ANIMATION_ID.to_string(),
            row: 5,
            frames: 1,
            fps: 1,
            tweens: Vec::new(),
            is_looping: false,
        }
    }

    pub fn default_death_back_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: DEATH_BACK_ANIMATION_ID.to_string(),
            row: 5,
            frames: 7,
            fps: 10,
            tweens: Vec::new(),
            is_looping: false,
        }
    }

    pub fn default_death_forward_animation() -> AnimationMetadata {
        AnimationMetadata {
            id: DEATH_FORWARD_ANIMATION_ID.to_string(),
            row: 6,
            frames: 7,
            fps: 10,
            tweens: Vec::new(),
            is_looping: false,
        }
    }
}

impl Default for PlayerAnimations {
    fn default() -> Self {
        PlayerAnimations {
            idle: Self::default_idle_animation(),
            moving: Self::default_move_animation(),
            jump: Self::default_jump_animation(),
            fall: Self::default_fall_animation(),
            crouch: Self::default_crouch_animation(),
            slide: Self::default_slide_animation(),
            death_back: Self::default_death_back_animation(),
            death_forward: Self::default_death_forward_animation(),
        }
    }
}

impl From<Vec<AnimationMetadata>> for PlayerAnimations {
    fn from(vec: Vec<AnimationMetadata>) -> Self {
        PlayerAnimations {
            idle: vec
                .iter()
                .find(|&anim| anim.id == *IDLE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            moving: vec
                .iter()
                .find(|&anim| anim.id == *MOVE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            jump: vec
                .iter()
                .find(|&anim| anim.id == *JUMP_ANIMATION_ID)
                .cloned()
                .unwrap(),
            fall: vec
                .iter()
                .find(|&anim| anim.id == *FALL_ANIMATION_ID)
                .cloned()
                .unwrap(),
            crouch: vec
                .iter()
                .find(|&anim| anim.id == *CROUCH_ANIMATION_ID)
                .cloned()
                .unwrap(),
            slide: vec
                .iter()
                .find(|&anim| anim.id == *SLIDE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death_back: vec
                .iter()
                .find(|&anim| anim.id == *DEATH_BACK_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death_forward: vec
                .iter()
                .find(|&anim| anim.id == *DEATH_FORWARD_ANIMATION_ID)
                .cloned()
                .unwrap(),
        }
    }
}

impl PlayerAnimations {
    pub fn into_vec(self) -> Vec<AnimationMetadata> {
        vec![
            self.idle,
            self.moving,
            self.jump,
            self.fall,
            self.crouch,
            self.slide,
            self.death_back,
            self.death_forward,
        ]
    }

    pub fn to_vec(&self) -> Vec<AnimationMetadata> {
        vec![
            self.idle.clone(),
            self.moving.clone(),
            self.jump.clone(),
            self.fall.clone(),
            self.crouch.clone(),
            self.slide.clone(),
            self.death_back.clone(),
            self.death_forward.clone(),
        ]
    }
}

pub fn update_player_animations(world: Arc<AtomicRefCell<World>>) {
    let mut world = AtomicRefCell::borrow_mut(world.as_ref());
    for (_, (player, inventory, body, drawable)) in
        world.query_mut::<(&Player, &mut PlayerInventory, &PhysicsBody, &mut Drawable)>()
    {
        let sprite_set = drawable.get_animated_sprite_set_mut().unwrap();

        sprite_set.flip_all_x(player.is_facing_left);
        sprite_set.flip_all_y(player.is_upside_down);

        let animation_id = match player.state {
            PlayerState::Dead => {
                if player.is_facing_left == player.damage_from_left {
                    DEATH_BACK_ANIMATION_ID
                } else {
                    DEATH_FORWARD_ANIMATION_ID
                }
            }
            PlayerState::Incapacitated => {
                // TODO: implement incapacitated
                unimplemented!();
            }
            PlayerState::Sliding => SLIDE_ANIMATION_ID,
            PlayerState::Crouching => CROUCH_ANIMATION_ID,
            _ => {
                if body.is_on_ground {
                    if !player.is_attacking && body.velocity.x != 0.0 {
                        MOVE_ANIMATION_ID
                    } else {
                        IDLE_ANIMATION_ID
                    }
                } else if body.velocity.y < 0.0 {
                    JUMP_ANIMATION_ID
                } else {
                    FALL_ANIMATION_ID
                }
            }
        };

        sprite_set.set_all(animation_id, false);

        let sprite = sprite_set.map.get(BODY_ANIMATED_SPRITE_ID).unwrap();
        let animation = sprite.current_animation();

        if let Some(tween) = animation.tweens.get(WEAPON_MOUNT_TWEEN_ID) {
            inventory.weapon_mount_offset = tween.current_translation;
        } else {
            inventory.weapon_mount_offset = Vec2::ZERO;
        }

        if let Some(tween) = animation.tweens.get(ITEM_MOUNT_TWEEN_ID) {
            inventory.item_mount_offset = tween.current_translation;
        } else {
            inventory.item_mount_offset = Vec2::ZERO;
        }

        if let Some(tween) = animation.tweens.get(HAT_MOUNT_TWEEN_ID) {
            inventory.hat_mount_offset = tween.current_translation;
        } else {
            inventory.hat_mount_offset = Vec2::ZERO;
        }
    }
}
