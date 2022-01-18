use fishsticks::GamepadContext;

use std::env;
use std::path::PathBuf;

use macroquad::{experimental::collections::storage, prelude::*};

pub mod config;
pub mod editor;
mod gui;
mod items;
pub mod json;
pub mod map;
pub mod math;
mod noise;
pub mod resources;
pub mod text;
#[macro_use]
pub mod error;
pub mod data;
pub mod debug;
pub mod ecs;
pub mod effects;
pub mod events;
pub mod game;
pub mod input;
pub mod network;
pub mod particles;
pub mod physics;
pub mod player;

mod channel;
mod sprite;
mod transform;

pub use input::*;
pub use physics::*;
pub use sprite::*;
pub use transform::*;

use editor::{Editor, EditorCamera, EditorInputScheme};

pub use error::{Error, ErrorKind, Result};

use map::{Map, MapLayerKind, MapObjectKind};

pub use channel::Channel;

pub use config::Config;
pub use items::Item;

pub use events::{dispatch_application_event, ApplicationEvent};

pub use game::{
    collect_local_input, start_music, stop_music, Game, GameCamera, GameInput, GameInputScheme,
};

pub use resources::Resources;

pub use player::PlayerEventParams;

pub use ecs::Owner;

use crate::game::GameMode;
use crate::network::Api;
use crate::resources::load_resources;
pub use effects::{
    ActiveEffectKind, ActiveEffectMetadata, PassiveEffectInstance, PassiveEffectMetadata,
};

pub type CollisionWorld = macroquad_platformer::World;

const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";

const WINDOW_TITLE: &str = "FishFight";

/// Exit to main menu
pub fn exit_to_main_menu() {
    ApplicationEvent::MainMenu.dispatch();
}

/// Quit to desktop
pub fn quit_to_desktop() {
    ApplicationEvent::Quit.dispatch()
}

fn window_conf() -> Conf {
    let config = Config::load(
        env::var(CONFIG_FILE_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.json");
                #[cfg(not(debug_assertions))]
                return PathBuf::from("./config.json");
            }),
    )
    .unwrap();

    storage::store(config.clone());

    Conf {
        window_title: WINDOW_TITLE.to_owned(),
        high_dpi: config.high_dpi,
        fullscreen: config.fullscreen,
        window_width: config.resolution.width,
        window_height: config.resolution.height,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    use events::iter_events;
    use gui::MainMenuResult;

    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "".to_string());

    rand::srand(0);

    load_resources(&assets_dir).await;

    {
        let gamepad_system = fishsticks::GamepadContext::init().unwrap();
        storage::store(gamepad_system);
    }

    'outer: loop {
        match gui::show_main_menu().await {
            MainMenuResult::LocalGame { map, players } => {
                let game = Game::new(GameMode::Local, map, &players);
                scene::add_node(game);

                start_music("fish_tide");
            }
            MainMenuResult::NetworkGame {
                host_id,
                map_resource: _,
                players: _,
            } => {
                let _is_host = Api::get_instance().is_own_id(host_id)?;

                // let game = NetworkGame::new(host_id, map_resource, &players)?;
                // scene::add_node(game);
                //
                // if is_host {
                //     let server = Server::new(DEFAULT_SERVER_PORT, &players)?;
                //     scene::add_node(server);
                // } else {
                //     let client = Client::new(host_id)?;
                //     scene::add_node(client);
                // }

                start_music("fish_tide");
            }
            MainMenuResult::Editor {
                input_scheme,
                is_new_map,
            } => {
                let map_resource = if is_new_map {
                    let res = gui::show_create_map_menu().await?;
                    if res.is_none() {
                        continue 'outer;
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
                let resources = storage::get::<Resources>();
                load_resources(&resources.assets_dir).await;
                continue 'outer;
            }
            MainMenuResult::Credits => {
                let resources = storage::get::<Resources>();
                start_music("thanks_for_all_the_fished");
                gui::show_game_credits(&resources.assets_dir).await;
                stop_music();
                continue 'outer;
            }
            MainMenuResult::Quit => {
                quit_to_desktop();
            }
        };

        'inner: loop {
            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    ApplicationEvent::ReloadResources => {
                        let resources = storage::get::<Resources>();
                        load_resources(&resources.assets_dir).await;
                    }
                    ApplicationEvent::MainMenu => break 'inner,
                    ApplicationEvent::Quit => break 'outer,
                }
            }

            {
                let mut gamepad_system = storage::get_mut::<GamepadContext>();
                gamepad_system.update()?;
            }

            next_frame().await;
        }

        scene::clear();
        stop_music();
    }

    Ok(())
}
