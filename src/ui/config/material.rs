use crate::ui::{
    config::{assets::TryAsRef, texture::TextureDescriptor},
    names::NamedHandle,
    render_view::{
        color_material::ColorMaterial, material::Material, texture::Texture,
        texture_material::TextureMaterial,
    },
    state::WgpuState,
};
use anyhow::Result;
use serde::Deserialize;
use std::{ops::Deref, rc::Rc};

use super::{handles::TextureName, WgpuResourceLoader};

#[derive(Deserialize, Debug)]
pub enum MaterialDescriptor {
    Texture(TextureMaterialDescriptor),
    Color(ColorMaterialDescriptor),
}

impl WgpuResourceLoader for MaterialDescriptor {
    type Output = Rc<dyn Material>;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        log::info!("load {}", self.name());
        if wgpu_state.store.contains_material(&self.name()) {
            log::info!("Hit wgpu store cache for {}", &self.name());
            return Ok(wgpu_state.store.get_material(&self.name().deref()).unwrap());
        }
        let material: Rc<dyn Material> = match self {
            MaterialDescriptor::Color(color) => {
                let material = ColorMaterial::new(
                    &wgpu_state.device,
                    color.name().deref(),
                    color.ambient,
                    color.diffuse,
                    color.specular,
                );
                Rc::new(material)
            }
            MaterialDescriptor::Texture(texture) => {
                let diffuse: Rc<Texture> = wgpu_state
                    .assets
                    .get(texture.diffuse_texture.clone())
                    .and_then(|desc| desc.try_as_ref())
                    .and_then(|descriptor: &TextureDescriptor| descriptor.load(wgpu_state))?;
                let normal: Rc<Texture> = wgpu_state
                    .assets
                    .get(texture.normal_texture.clone())
                    .and_then(|desc| desc.try_as_ref())
                    .and_then(|descriptor: &TextureDescriptor| descriptor.load(wgpu_state))?;

                Rc::new(TextureMaterial::new(
                    &wgpu_state.device,
                    texture.name().deref(),
                    diffuse.as_ref(),
                    normal.as_ref(),
                ))
            }
        };
        wgpu_state.store.add_material(material.clone());
        Ok(material)
    }
}

#[derive(Deserialize, Debug)]
pub struct ColorMaterialDescriptor {
    pub(crate) name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

impl ColorMaterialDescriptor {
    pub fn _new_(name: String, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) -> Self {
        Self {
            name,
            ambient,
            diffuse,
            specular,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TextureMaterialDescriptor {
    pub(crate) name: String,
    pub diffuse_texture: TextureName,
    pub normal_texture: TextureName,
}

//TODO: delete after made better
impl TextureMaterialDescriptor {
    pub fn _new_(name: String, diffuse_texture: TextureName, normal_texture: TextureName) -> Self {
        Self {
            name,
            diffuse_texture,
            normal_texture,
        }
    }
}
