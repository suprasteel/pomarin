use crate::render::{
    scene::texture::{Texture, TextureKind},
    state::WgpuState,
};
use anyhow::Result;
use serde::Deserialize;
use std::{fmt::Display, path::PathBuf, rc::Rc};

use super::WgpuResourceLoader;

// TODO: check both are used
#[derive(Deserialize, Debug, Clone)]
pub struct TextureDescriptor {
    pub(crate) name: String,
    path: PathBuf,
    kind: TextureKind,
}

impl TextureDescriptor {
    pub fn _new_(name: String, path: PathBuf, kind: TextureKind) -> Self {
        Self { name, path, kind }
    }
}

impl WgpuResourceLoader for TextureDescriptor {
    type Output = Rc<Texture>;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        log::info!("Load {}", self.name);
        let directory = PathBuf::from(wgpu_state.settings.textures_directory.to_string());

        log::info!(
            "Load texture file from : {:?} (textures directory is: {:?})",
            self.path,
            directory
        );

        if wgpu_state.store.contains_texture(&self.name) {
            return Ok(wgpu_state
                .store
                .get_texture(&self.name)
                .expect("Impossible err 2"));
        }
        let is_normal_map = self.kind == TextureKind::Normal;
        let texture = Texture::load(
            &wgpu_state.device,
            &wgpu_state.queue,
            directory.join(&self.path),
            is_normal_map,
        )?;
        let texture = Rc::new(texture);
        wgpu_state.store.add_texture(&self.name, texture.clone());
        Ok(texture.clone())
    }
}

impl Display for TextureDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let k = if self.kind == TextureKind::Diffuse {
            "diffuse"
        } else {
            "normal"
        };
        write!(
            f,
            "TextureDescriptor:\"{}\" of type {} from {:?}",
            self.name, k, self.path
        )
    }
}
