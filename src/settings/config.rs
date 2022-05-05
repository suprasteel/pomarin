use std::{fs, path::PathBuf};

use serde::Deserialize;
use winit::dpi::{PhysicalSize, Size};

static CONF_PATH: &'static str = env!("APP_CONF_FILE_PATH");

/// Initial window configuration
#[derive(Copy, Clone, Deserialize, Debug)]
pub struct WindowConfig {
    pub height: u32,
    pub width: u32,
    pub maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            height: 300,
            width: 400,
            maximized: false,
        }
    }
}

impl From<WindowConfig> for Size {
    fn from(wconfig: WindowConfig) -> Self {
        Size::Physical(PhysicalSize {
            height: wconfig.height,
            width: wconfig.width,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct ResourcesConfig {
    /// textures as images
    pub textures_directory: String,
    /// meshes as (.obj) files
    pub meshes_directory: String,
    /// file path of the list of models (use mesh + material ...)
    pub models_cfg: String,
    /// file path of the list of meshes
    pub meshes_cfg: String,
    /// list of resources to load (textures files from texture_directory, meshes files from
    pub materials_cfg: String,
    /// file path of the list of textures
    pub textures_cfg: String,
}

fn tostring(pathbuf: PathBuf) -> String {
    pathbuf.into_os_string().into_string().unwrap()
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        let out_dir: PathBuf = PathBuf::from(env!("OUT_DIR").to_string());
        Self {
            textures_directory: tostring(out_dir.join("textures")),
            meshes_directory: tostring(out_dir.join("meshes")),
            models_cfg: tostring(out_dir.join("models.ron")),
            meshes_cfg: tostring(out_dir.join("meshes.ron")),
            materials_cfg: tostring(out_dir.join("materials.ron")),
            textures_cfg: tostring(out_dir.join("textures.ron")),
        }
    }
}

/// Initial application configuration
#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub resources: ResourcesConfig,
}

/// Load configuration from default local file.
/// In case of failure, return the default configuration
pub fn load_conf() -> AppConfig {
    log::info!("Loading configuration from {}", CONF_PATH);
    match fs::read_to_string(CONF_PATH) {
        Ok(string) => match ron::from_str(&string) {
            Ok(content) => content,
            Err(e) => {
                log::warn!("Application configuration parsing failure");
                log::warn!("{}", e);
                AppConfig::default()
            }
        },
        Err(e) => {
            log::warn!("Configuration file not found");
            log::warn!("{}", e);
            AppConfig::default()
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: Default::default(),
            resources: Default::default(),
        }
    }
}
