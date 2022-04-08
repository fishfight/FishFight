use fishsticks::GamepadContext;
use hecs::{DynamicQuery, Entity, World};

use core::lua::wrapped_types::{ColorLua, RectLua, SoundLua, Texture2DLua, Vec2Lua};
use core::lua::CloneComponent;
use std::env;
use std::path::PathBuf;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

pub mod debug;
pub mod ecs;
pub mod editor;
pub mod effects;
pub mod events;
pub mod game;
mod gui;
mod items;
pub mod json;
pub mod map;
pub mod network;
pub mod particles;
pub mod physics;
pub mod player;
pub mod resources;

pub mod drawables;

mod lua;

pub use drawables::*;
pub use physics::*;

use editor::{Editor, EditorCamera, EditorInputScheme};

use map::{Map, MapLayerKind, MapObjectKind};

use core::Result;
use core::{network::Api, Transform};

pub use core::Config;
pub use items::Item;

pub use events::{dispatch_application_event, ApplicationEvent};

pub use game::{start_music, stop_music, Game, GameCamera};

pub use resources::Resources;

pub use player::PlayerEvent;

pub use ecs::Owner;

use crate::effects::active::projectiles::{Projectile, Rectangle};
use crate::effects::active::triggered::TriggeredEffect;
use crate::effects::active::{
    ActiveEffectKindCircleCollider, ActiveEffectKindProjectile, ActiveEffectKindRectCollider,
    ActiveEffectKindTriggeredEffect, ProjectileKind,
};
use crate::effects::passive::init_passive_effects;
use crate::effects::TriggeredEffectTrigger;
use crate::game::GameMode;
use crate::items::{ItemDepleteBehavior, ItemDropBehavior, ItemMetadata, Weapon};
use crate::lua::ActorLua;
use crate::particles::{ParticleEmitter, ParticleEmitterMetadata, Particles};
use crate::player::{Player, PlayerEventKind, PlayerInventory, PlayerState};
use crate::resources::load_resources;
pub use effects::{
    ActiveEffectKind, ActiveEffectMetadata, PassiveEffectInstance, PassiveEffectMetadata,
};

pub type CollisionWorld = macroquad_platformer::World;

const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";
const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const MODS_DIR_ENV_VAR: &str = "FISHFIGHT_MODS";

const WINDOW_TITLE: &str = "Fish Fight";

/// Exit to main menu
pub fn exit_to_main_menu() {
    ApplicationEvent::MainMenu.dispatch();
}

/// Quit to desktop
pub fn quit_to_desktop() {
    ApplicationEvent::Quit.dispatch()
}

/// Reload resources
pub fn reload_resources() {
    ApplicationEvent::ReloadResources.dispatch()
}

fn window_conf() -> Conf {
    let path = env::var(CONFIG_FILE_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
            #[cfg(not(debug_assertions))]
            return PathBuf::from("./config.toml");
        });

    let config = Config::load(&path).unwrap();

    storage::store(config.clone());

    Conf {
        window_title: WINDOW_TITLE.to_owned(),
        high_dpi: config.window.is_high_dpi,
        fullscreen: config.window.is_fullscreen,
        window_width: config.window.width as i32,
        window_height: config.window.height as i32,
        ..Default::default()
    }
}

/// Returns `true` if the outer game loop should continue;
#[cfg(not(feature = "ultimate"))]
async fn init_game() -> Result<bool> {
    use gui::MainMenuResult;

    match gui::show_main_menu().await {
        MainMenuResult::LocalGame { map, players } => {
            let game = Game::new(GameMode::Local, *map, &players)?;
            scene::add_node(game);

            start_music("fish_tide");
        }
        MainMenuResult::Editor {
            input_scheme,
            is_new_map,
        } => {
            let map_resource = if is_new_map {
                let res = gui::show_create_map_menu().await?;
                if res.is_none() {
                    return Ok(true);
                }

                res.unwrap()
            } else {
                gui::show_select_map_menu().await
            };

            let position = map_resource.map.get_size() * 0.5;

            scene::add_node(EditorCamera::new(position));
            scene::add_node(Editor::new(input_scheme, map_resource));
        }
        MainMenuResult::ReloadResources => {
            reload_resources();
            return Ok(true);
        }
        MainMenuResult::Credits => {
            let resources = storage::get::<Resources>();
            start_music("thanks_for_all_the_fished");
            gui::show_game_credits(&resources.assets_dir).await;
            stop_music();
            return Ok(true);
        }
        MainMenuResult::Quit => {
            quit_to_desktop();
        }
    };

    Ok(false)
}

#[cfg(feature = "ultimate")]
async fn init_game() -> Result<bool> {
    use core::input::GameInputScheme;
    use core::network::Api;

    use crate::player::{PlayerControllerKind, PlayerParams};

    let player_ids = vec!["1".to_string(), "2".to_string()];

    Api::init::<ultimate::UltimateApiBackend>(&player_ids[0], true).await?;

    let (map, mut characters) = {
        let resources = storage::get::<Resources>();

        let map = resources.maps.first().map(|res| res.map.clone()).unwrap();

        let characters = vec![
            resources.player_characters.get("pescy").cloned().unwrap(),
            resources.player_characters.get("sharky").cloned().unwrap(),
        ];

        (map, characters)
    };

    let players = vec![
        PlayerParams {
            index: 0,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft).into(),
            character: characters.pop().unwrap(),
        },
        PlayerParams {
            index: 1,
            controller: PlayerControllerKind::Network(player_ids[1].clone()).into(),
            character: characters.pop().unwrap(),
        },
    ];

    let game = Game::new(GameMode::NetworkHost, map, &players)?;
    scene::add_node(game);

    start_music("fish_tide");

    Ok(false)
}

#[macroquad::main(window_conf)]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let types = tealr::TypeWalker::new()
        .process_type::<World>()
        .process_type::<ParticleEmitterMetadata>()
        .process_type::<PassiveEffectMetadata>()
        .process_type::<PlayerEventKind>()
        .process_type::<Vec2Lua>()
        .process_type::<ActorLua>()
        .process_type::<RectLua>()
        .process_type::<ColorLua>()
        .process_type::<Entity>()
        .process_type::<TriggeredEffectTrigger>()
        .process_type::<ActiveEffectMetadata>()
        .process_type::<ItemDropBehavior>()
        .process_type::<ItemDepleteBehavior>()
        .process_type::<Transform>()
        .process_type::<PhysicsBody>()
        .process_type::<RigidBody>()
        .process_type::<Projectile>()
        .process_type::<TriggeredEffect>()
        .process_type::<Item>()
        .process_type::<Owner>()
        .process_type::<PlayerInventory>()
        .process_type::<Player>()
        .process_type::<PlayerState>()
        .process_type::<PassiveEffectInstance>()
        .process_type::<ProjectileKind>()
        .process_type::<PlayerEvent>()
        .process_type::<Texture2DLua>()
        .process_type::<ItemMetadata>()
        .process_type::<Weapon>()
        .process_type::<Animation>()
        .process_type::<Keyframe>()
        .process_type::<AnimatedSpriteParams>()
        .process_type::<AnimatedSprite>()
        .process_type::<QueuedAnimationAction>()
        .process_type::<Tween>()
        .process_type::<DynamicQuery>()
        .process_type::<ParticleEmitter>()
        .process_type::<ActiveEffectKind>()
        .process_type::<effects::active::projectiles::Circle>()
        .process_type::<Rectangle>()
        .process_type::<effects::active::projectiles::SpriteProjectile>()
        .process_type::<ActiveEffectKindCircleCollider>()
        .process_type::<ActiveEffectKindRectCollider>()
        .process_type::<ActiveEffectKindTriggeredEffect>()
        .process_type::<ActiveEffectKindProjectile>()
        .process_type::<SoundLua>()
        .process_type::<AnimationMetadata>()
        .process_type::<AnimatedSpriteMetadata>()
        .process_type::<CloneComponent<tealr::mlu::generics::X>>()
        .process_type::<TweenMetadata>()
        .process_type::<Drawable>()
        .process_type::<SpriteSet>()
        .process_type::<Sprite>()
        .process_type::<SpriteParams>()
        .process_type::<crate::drawables::DrawableKind>()
        .process_type::<AnimatedSpriteSet>()
        .process_type::<crate::lua::TypeComponentContainer>()
        .process_type_as_marker::<CloneComponent<tealr::mlu::generics::X>>()
        .process_type_as_marker::<Owner>()
        .process_type_as_marker::<ParticleEmitter>()
        .process_type_as_marker::<RectLua>()
        .process_type_as_marker::<Animation>()
        .process_type_as_marker::<AnimatedSprite>()
        .process_type_as_marker::<crate::drawables::DrawableKind>()
        .process_type_as_marker::<AnimatedSpriteSet>()
        .process_type_as_marker::<Sprite>();
    println!("time to generate the json files");
    std::fs::write("./test.json", serde_json::to_string_pretty(&types).unwrap()).unwrap();
    std::fs::write("./test.d.tl", types.generate_global("test").unwrap()).unwrap();
    println!("Wrote all!");
    // println!("Starting embedded lua test");
    // core::test::test()?;
    // println!("Ended embedded lua test");
    use events::iter_events;

    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());
    let mods_dir = env::var(MODS_DIR_ENV_VAR).unwrap_or_else(|_| "./mods".to_string());

    rand::srand(0);

    load_resources(&assets_dir, &mods_dir).await?;

    {
        let gamepad_context = fishsticks::GamepadContext::init().unwrap();
        storage::store(gamepad_context);
    }

    {
        let particles = Particles::new();
        storage::store(particles);
    }

    init_passive_effects();

    'outer: loop {
        if init_game().await? {
            continue 'outer;
        }

        'inner: loop {
            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    ApplicationEvent::ReloadResources => {
                        load_resources(&assets_dir, &mods_dir).await?;
                    }
                    ApplicationEvent::MainMenu => break 'inner,
                    ApplicationEvent::Quit => break 'outer,
                }
            }

            {
                let mut gamepad_context = storage::get_mut::<GamepadContext>();
                gamepad_context.update()?;
            }

            next_frame().await;
        }

        scene::clear();

        stop_music();
    }

    Api::close().await?;

    Ok(())
}
