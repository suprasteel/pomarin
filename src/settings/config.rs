use std::fs;

use serde::Deserialize;
use winit::dpi::{PhysicalSize, Size};

static CONF_PATH: &'static str = env!("APP_CONF_FILE_PATH");

/// Initial window configuration
#[derive(Copy, Clone, Deserialize)]
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

#[derive(Deserialize)]
pub struct ResourcesConfig {
    /// textures as images
    pub textures_directory: String,
    /// meshes as .obj files
    pub meshes_directory: String,
    /// list of models (use mesh + material ...)
    pub models_description_cfg: String,
    /// list of resources to load (textures files from texture_directory, meshes files from
    /// meshes_directory
    pub materials_description_cfg: String,
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        Self {
            textures_directory: env!("OUT_DIR").to_string() + "./textures",
            meshes_directory: env!("OUT_DIR").to_string() + "./meshes",
            models_description_cfg: env!("OUT_DIR").to_string() + "./config/models.cfg",
            materials_description_cfg: env!("OUT_DIR").to_string() + "./config/materials.cfg",
        }
    }
}

/// Initial application configuration
#[derive(Deserialize)]
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
