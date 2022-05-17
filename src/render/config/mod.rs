//! A module to manage assets reading, parsing and wgpu resources initialisation.

use super::state::WgpuState;
use anyhow::Result;

/// Module to handle all kind of asset as one type
pub mod assets;
/// Module defining geometry configuration data
pub mod geometry;
/// Module defining asset kind names as strongly typed
pub mod handles;
/// Module defining material configuration data
pub mod material;
/// Module defining mesh configuration data
pub mod mesh;
/// Module defining model configuration data
pub mod model;
/// Module defining texture configuration data
pub mod texture;
/// Module defining vertex configuration data
pub mod vertex;

/// Structs implementing this trait have to provide a wgpu resource from themselves.
///
/// This trait is to be implemented by assets descriptor to build wgpu resources (buffers, uniforms, bindings).
///
/// This trait makes the implementor aware of the wgpu state.
///
/// The wgpu state has a buffer store to retain buffers or other wgpu resources (check the Store)
pub trait WgpuResourceLoader {
    type Output;

    /// Load a wgpu resource from the implementor.
    /// This method should save in the store the built resources if not present yet.
    /// Do nothing if the resource is already available in the store.
    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output>;
}

pub mod utils {
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::Path;

    use crate::{app::config::ResourcesConfig, render::config::assets::AssetsDescriptors};

    use super::{
        material::MaterialDescriptor, mesh::MeshDescriptor, model::ModelDescriptor,
        texture::TextureDescriptor,
    };

    /// Given a ResourcesConfig, loads all available assets.
    ///
    /// The ResourcesConfig has files paths to the configuration file containings our assets descriptors.
    pub fn load_assets(config: &ResourcesConfig) -> Result<AssetsDescriptors> {
        let mut ad = AssetsDescriptors::new();

        let textures_desc = read_textures_descriptors(&config.textures_cfg)?;
        textures_desc.into_iter().for_each(|t| ad.push(t));

        let materials_desc = read_materials_descriptors(&config.materials_cfg)?;
        materials_desc.into_iter().for_each(|t| ad.push(t));

        let meshes_desc = read_mesh_descriptors(&config.meshes_cfg)?;
        meshes_desc.into_iter().for_each(|t| ad.push(t));

        let model_desc = read_models_descriptors(&config.models_cfg)?;
        model_desc.into_iter().for_each(|t| ad.push(t));

        Ok(ad)
    }

    pub fn read_materials_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<MaterialDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading materials file")?;
        let list: Vec<MaterialDescriptor> =
            ron::from_str(&string_content).context("parsing materials")?;
        Ok(list)
    }
    pub fn read_mesh_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<MeshDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading meshes file")?;
        let list: Vec<MeshDescriptor> = ron::from_str(&string_content).context("parsing meshes")?;
        Ok(list)
    }
    pub fn read_textures_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<TextureDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading textures file")?;
        let list: Vec<TextureDescriptor> =
            ron::from_str(&string_content).context("parsing textures")?;
        Ok(list)
    }
    pub fn read_models_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<ModelDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading models file")?;
        let list: Vec<ModelDescriptor> =
            ron::from_str(&string_content).context("parsing models")?;
        Ok(list)
    }
}
