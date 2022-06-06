use fishsticks::GamepadContext;

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
pub mod utils;

pub mod drawables;

pub use drawables::*;
pub use physics::*;

use editor::EditorInputScheme;

use map::{Map, MapLayerKind, MapObjectKind};

use core::network::Api;
use core::Result;

pub use core::Config;
pub use items::Item;

pub use events::{dispatch_application_event, ApplicationEvent};

pub use game::{start_music, stop_music, Game, GameCamera};

pub use resources::Resources;

pub use player::PlayerEvent;

pub use ecs::Owner;

use crate::effects::passive::init_passive_effects;
use crate::game::GameMode;
use crate::particles::Particles;
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

    use crate::editor::{Editor, EditorNode};

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

            scene::add_node(EditorNode::new(Editor::new(map_resource), input_scheme));
        }
        MainMenuResult::ReloadResources => {
            reload_resources();
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
                        break 'inner;
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
