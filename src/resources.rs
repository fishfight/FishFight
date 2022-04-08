use std::{collections::HashMap, fs, os::unix::prelude::OsStrExt, path::Path};

use macroquad::{
    audio::{load_sound, Sound},
    experimental::{collections::storage, coroutines::start_coroutine},
    prelude::*,
};

use ff_particles::EmitterConfig;

use serde::{Deserialize, Serialize};

use core::text::ToStringHelper;
use core::{
    data::{deserialize_json_bytes, deserialize_json_file},
    lua::init_lua,
};
use core::{error::ErrorKind, lua::load_lua};
use core::{formaterr, Result};

use crate::map::DecorationMetadata;
use crate::{gui::GuiResources, lua::register_types};

use crate::player::PlayerCharacterMetadata;
use crate::{items::MapItemMetadata, map::Map};

const PARTICLE_EFFECTS_DIR: &str = "particle_effects";
const SOUNDS_FILE: &str = "sounds";
const MUSIC_FILE: &str = "music";
const TEXTURES_FILE: &str = "textures";
const IMAGES_FILE: &str = "images";
const MAPS_FILE: &str = "maps";
const DECORATION_FILE: &str = "decoration";
const ITEMS_FILE: &str = "items";
const PLAYER_CHARACTERS_FILE: &str = "player_characters";

const RESOURCE_FILES_EXTENSION: &str = "json";

pub const MAP_EXPORTS_DEFAULT_DIR: &str = "maps";
pub const MAP_EXPORTS_EXTENSION: &str = "json";
pub const MAP_EXPORT_NAME_MIN_LEN: usize = 1;

pub const MAP_PREVIEW_PLACEHOLDER_PATH: &str = "maps/no_preview.png";
pub const MAP_PREVIEW_PLACEHOLDER_ID: &str = "map_preview_placeholder";

const ACTIVE_MODS_FILE_NAME: &str = "active_mods";
const MOD_FILE_NAME: &str = "fishfight_mod";

#[derive(Serialize, Deserialize)]
struct ParticleEffectMetadata {
    id: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct SoundMetadata {
    id: String,
    path: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureKind {
    Background,
    Tileset,
    Spritesheet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<TextureKind>,
    #[serde(
        default,
        alias = "sprite_size",
        with = "core::json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_size: Option<Vec2>,
    #[serde(
        default = "core::json::default_filter_mode",
        with = "core::json::FilterModeDef"
    )]
    pub filter_mode: FilterMode,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct TextureResource {
    pub texture: Texture2D,
    pub meta: TextureMetadata,
}

impl TextureResource {
    pub fn frame_size(&self) -> Vec2 {
        self.meta.frame_size.unwrap_or(self.meta.size)
    }
}

impl From<&TextureResource> for Texture2D {
    fn from(res: &TextureResource) -> Self {
        res.texture
    }
}

impl From<TextureResource> for Texture2D {
    fn from(res: TextureResource) -> Self {
        res.texture
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub image: Image,
    pub meta: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub preview_path: String,
    #[serde(default, skip_serializing_if = "core::json::is_false")]
    pub is_tiled_map: bool,
    #[serde(default, skip_serializing_if = "core::json::is_false")]
    pub is_user_map: bool,
}

#[derive(Debug, Clone)]
pub struct MapResource {
    pub map: Map,
    pub preview: Texture2D,
    pub meta: MapMetadata,
}

// TODO: Add an optional requirement for all resource files (for when loading games main resources)
async fn load_resources_from<P: AsRef<Path>>(path: P, resources: &mut Resources) -> Result<()> {
    let path = path.as_ref();

    {
        let particle_effects_file_path = path
            .join(PARTICLE_EFFECTS_DIR)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&particle_effects_file_path.to_string_helper()).await {
            let metadata: Vec<ParticleEffectMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let file_path = path.join(&meta.path);

                let cfg: EmitterConfig = deserialize_json_file(&file_path).await?;

                resources.particle_effects.insert(meta.id, cfg);
            }
        }
    }

    {
        let sounds_file_path = path
            .join(SOUNDS_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&sounds_file_path.to_string_helper()).await {
            let metadata: Vec<SoundMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let file_path = path.join(meta.path);

                let sound = load_sound(&file_path.to_string_helper()).await?;

                resources.sounds.insert(meta.id, sound);
            }
        }
    }

    {
        let music_file_path = path
            .join(MUSIC_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&music_file_path.to_string_helper()).await {
            let metadata: Vec<SoundMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let file_path = path.join(meta.path);

                let sound = load_sound(&file_path.to_string_helper()).await?;

                resources.music.insert(meta.id, sound);
            }
        }
    }

    {
        let textures_file_path = path
            .join(TEXTURES_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&textures_file_path.to_string_helper()).await {
            let metadata: Vec<TextureMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let file_path = path.join(&meta.path);

                let texture = load_texture(&file_path.to_string_helper()).await?;
                texture.set_filter(meta.filter_mode);

                let size = vec2(texture.width(), texture.height());

                let key = meta.id.clone();

                let meta = TextureMetadata { size, ..meta };

                #[cfg(debug_assertions)]
                if meta.frame_size.is_none()
                    && meta.kind.is_some()
                    && meta.kind.unwrap() == TextureKind::Spritesheet
                {
                    println!(
                        "WARNING: The texture '{}' is a spritesheet but no frame size has been set",
                        &meta.id
                    );
                }

                let res = TextureResource { texture, meta };

                resources.textures.insert(key, res);
            }
        }
    }

    {
        let images_file_path = path
            .join(IMAGES_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&images_file_path.to_string_helper()).await {
            let metadata: Vec<ImageMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let file_path = path.join(&meta.path);

                let image = load_image(&file_path.to_string_helper()).await?;

                let key = meta.id.clone();

                let meta = ImageMetadata {
                    size: vec2(image.width() as f32, image.height() as f32),
                    ..meta
                };

                let res = ImageResource { image, meta };

                resources.images.insert(key, res);
            }
        }
    }

    {
        let maps_file_path = path
            .join(MAPS_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&maps_file_path.to_string_helper()).await {
            let metadata: Vec<MapMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                let map_path = path.join(&meta.path);
                let preview_path = path.join(&meta.preview_path);

                let map = if meta.is_tiled_map {
                    Map::load_tiled(map_path, None).await?
                } else {
                    Map::load(map_path).await?
                };

                let preview = load_texture(&preview_path.to_string_helper()).await?;

                let res = MapResource { map, preview, meta };

                resources.maps.push(res)
            }
        }
    }

    {
        let decoration_file_path = path
            .join(DECORATION_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&decoration_file_path.to_string_helper()).await {
            let decoration_paths: Vec<String> = deserialize_json_bytes(&bytes)?;

            for decoration_path in decoration_paths {
                let path = path.join(&decoration_path);

                let params: DecorationMetadata = deserialize_json_file(&path).await?;

                resources.decoration.insert(params.id.clone(), params);
            }
        }
    }

    {
        let items_file_path = path
            .join(ITEMS_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&items_file_path.to_string_helper()).await {
            let item_paths: Vec<String> = deserialize_json_bytes(&bytes)?;

            for item_path in item_paths {
                let path = path.join(&item_path);

                let params: MapItemMetadata = deserialize_json_file(&path).await?;

                resources.items.insert(params.id.clone(), params);
            }
        }
    }

    {
        let path = path
            .join(PLAYER_CHARACTERS_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        if let Ok(bytes) = load_file(&path.to_string_helper()).await {
            let metadata: Vec<PlayerCharacterMetadata> = deserialize_json_bytes(&bytes)?;

            for meta in metadata {
                resources.player_characters.insert(meta.id.clone(), meta);
            }
        }
    };

    Ok(())
}

pub struct Resources {
    pub assets_dir: String,
    pub mods_dir: String,

    pub loaded_mods: Vec<ModMetadata>,

    pub particle_effects: HashMap<String, EmitterConfig>,
    pub sounds: HashMap<String, Sound>,
    pub music: HashMap<String, Sound>,
    pub textures: HashMap<String, TextureResource>,
    pub images: HashMap<String, ImageResource>,
    pub maps: Vec<MapResource>,
    pub decoration: HashMap<String, DecorationMetadata>,
    pub items: HashMap<String, MapItemMetadata>,
    pub player_characters: HashMap<String, PlayerCharacterMetadata>,
    pub lua: hv_lua::Lua,
}

impl Resources {
    pub async fn new<P: AsRef<Path>>(assets_dir: P, mods_dir: P) -> Result<Resources> {
        let assets_dir = assets_dir.as_ref();
        let mods_dir = mods_dir.as_ref();
        let lua = init_lua(mods_dir, register_types).unwrap();
        let mut resources = Resources {
            assets_dir: assets_dir.to_string_helper(),
            mods_dir: mods_dir.to_string_helper(),
            loaded_mods: Vec::new(),
            particle_effects: HashMap::new(),
            sounds: HashMap::new(),
            music: HashMap::new(),
            textures: HashMap::new(),
            decoration: HashMap::new(),
            images: HashMap::new(),
            maps: Vec::new(),
            items: HashMap::new(),
            player_characters: HashMap::new(),
            lua,
        };

        load_resources_from(assets_dir, &mut resources).await?;

        load_mods(mods_dir, &mut resources).await?;

        Ok(resources)
    }

    pub fn create_map(
        &self,
        name: &str,
        description: Option<&str>,
        tile_size: Vec2,
        grid_size: UVec2,
    ) -> Result<MapResource> {
        let description = description.map(|str| str.to_string());

        let map_path = Path::new(MAP_EXPORTS_DEFAULT_DIR)
            .join(map_name_to_filename(name))
            .with_extension(MAP_EXPORTS_EXTENSION);

        let path = map_path.to_string_helper();

        let preview_path = Path::new(MAP_PREVIEW_PLACEHOLDER_PATH).to_string_helper();

        let meta = MapMetadata {
            name: name.to_string(),
            description,
            path,
            preview_path,
            is_tiled_map: false,
            is_user_map: true,
        };

        let map = Map::new(tile_size, grid_size);

        let preview = {
            let res = self.textures.get(MAP_PREVIEW_PLACEHOLDER_ID).unwrap();
            res.texture
        };

        Ok(MapResource { map, preview, meta })
    }

    pub fn save_map(&mut self, map_resource: &MapResource) -> Result<()> {
        let assets_path = Path::new(&self.assets_dir);
        let export_path = assets_path.join(&map_resource.meta.path);

        if export_path.exists() {
            let mut i = 0;
            while i < self.maps.len() {
                let res = &self.maps[i];
                if res.meta.path == map_resource.meta.path {
                    if res.meta.is_user_map {
                        self.maps.remove(i);
                        break;
                    } else {
                        return Err(formaterr!(
                            ErrorKind::General,
                            "Resources: The path '{}' is in use and it is not possible to overwrite. Please choose a different map name",
                            &map_resource.meta.path,
                        ));
                    }
                }

                i += 1;
            }
        }

        map_resource.map.save(export_path)?;

        self.maps.push(map_resource.clone());
        self.save_maps_file()?;

        Ok(())
    }

    pub fn delete_map(&mut self, index: usize) -> Result<()> {
        let map_resource = self.maps.remove(index);

        let path = Path::new(&self.assets_dir).join(&map_resource.meta.path);

        fs::remove_file(path)?;

        self.save_maps_file()?;

        Ok(())
    }

    fn save_maps_file(&self) -> Result<()> {
        let maps_file_path = Path::new(&self.assets_dir)
            .join(MAPS_FILE)
            .with_extension(RESOURCE_FILES_EXTENSION);

        let metadata: Vec<MapMetadata> = self.maps.iter().map(|res| res.meta.clone()).collect();

        let str = serde_json::to_string_pretty(&metadata)?;
        fs::write(maps_file_path, &str)?;

        Ok(())
    }
}

pub fn map_name_to_filename(name: &str) -> String {
    name.replace(' ', "_").replace('.', "_").to_lowercase()
}

pub fn is_valid_map_file_name(file_name: &str) -> bool {
    if file_name.len() - MAP_EXPORTS_EXTENSION.len() > MAP_EXPORT_NAME_MIN_LEN {
        if let Some(extension) = Path::new(file_name).extension() {
            return extension == MAP_EXPORTS_EXTENSION;
        }
    }

    false
}

pub fn is_valid_map_export_path<P: AsRef<Path>>(path: P, should_overwrite: bool) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
        if is_valid_map_file_name(&file_name.to_string_helper()) {
            let resources = storage::get::<Resources>();

            let res = resources
                .maps
                .iter()
                .find(|res| Path::new(&res.meta.path) == path);

            if let Some(res) = res {
                return res.meta.is_user_map && should_overwrite;
            }

            return true;
        }
    }

    false
}

#[cfg(target_arch = "wasm32")]
pub async fn load_resources(assets_dir: &str, mods_dir: &str) -> Result<()> {
    {
        let resources = Resources::new(assets_dir, mods_dir).await?;
        storage::store(resources);
    }

    {
        let gui_resources = GuiResources::new();
        storage::store(gui_resources);
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_resources(assets_dir: &str, mods_dir: &str) -> Result<()> {
    let assets_loading = start_coroutine({
        let assets_dir = assets_dir.to_string();
        let mods_dir = mods_dir.to_string();
        async move {
            let resources = match Resources::new(&assets_dir, &mods_dir).await {
                Ok(val) => val,
                Err(err) => panic!("{}: {}", err.kind().as_str(), err),
            };

            storage::store(resources);
        }
    });

    while !assets_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading assets {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );

        next_frame().await;
    }

    {
        let gui_resources = GuiResources::new();
        storage::store(gui_resources);
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub kind: ModKind,
    #[serde(default)]
    pub dependencies: Vec<DependencyMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetadata {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModKind {
    DataOnly,
    Full,
}

impl Default for ModKind {
    fn default() -> Self {
        ModKind::Full
    }
}

// TODO: Better version checks
async fn load_mods<P: AsRef<Path>>(mods_dir: P, resources: &mut Resources) -> Result<()> {
    let mods_dir = mods_dir.as_ref();

    let active_mods_file_path = mods_dir
        .join(ACTIVE_MODS_FILE_NAME)
        .with_extension(RESOURCE_FILES_EXTENSION);

    let mod_dirs: Vec<String> = deserialize_json_file(active_mods_file_path).await?;

    for mod_dir in mod_dirs.iter() {
        let mod_dir_path = mods_dir.join(mod_dir);

        let mod_file_path = mod_dir_path
            .join(MOD_FILE_NAME)
            .with_extension(RESOURCE_FILES_EXTENSION);

        let meta: ModMetadata = deserialize_json_file(mod_file_path).await?;

        let mut has_game_version_mismatch = false;

        if let Some(req_version) = &meta.game_version {
            if *req_version != env!("CARGO_PKG_VERSION") {
                has_game_version_mismatch = true;

                #[cfg(debug_assertions)]
                println!(
                    "Loading mod {} (v{}) failed: Game version requirement mismatch (v{})",
                    &meta.id, &meta.version, req_version
                );
            }
        }

        if !has_game_version_mismatch {
            let mut has_unmet_dependencies = false;

            for dependency in &meta.dependencies {
                let res = resources
                    .loaded_mods
                    .iter()
                    .find(|&meta| meta.id == dependency.id && meta.version == dependency.version);

                if res.is_none() {
                    has_unmet_dependencies = true;

                    #[cfg(debug_assertions)]
                    println!(
                        "Loading mod {} (v{}) failed: Unmet dependency {} (v{})",
                        &meta.id, &meta.version, &dependency.id, &dependency.version
                    );

                    break;
                }
            }

            if !has_unmet_dependencies {
                let path = mod_dir_path.file_name().unwrap().as_bytes().to_owned();
                load_resources_from(mod_dir_path, resources).await?;
                if meta.kind == ModKind::Full {
                    load_lua(meta.id.to_owned(), path, &resources.lua).unwrap();
                }
                #[cfg(debug_assertions)]
                println!("Loaded mod {} (v{})", &meta.id, &meta.version);

                resources.loaded_mods.push(meta);
            }
        }
    }

    Ok(())
}
