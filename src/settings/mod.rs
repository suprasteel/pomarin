pub mod assets;
pub mod config;

pub mod utils {
    use std::{fs, path::Path};

    use anyhow::{Context, Result};

    use super::config::ResourcesConfig;
    use crate::{
        settings::assets::AssetsDescriptors,
        ui::{
            material::MaterialDescriptor, mesh::MeshDescriptor, model::ModelDescriptor,
            texture::TextureDescriptor,
        },
    };

    //TODO: list errors without blocking all
    pub fn load_assets(config: &ResourcesConfig) -> Result<AssetsDescriptors> {
        let mut ad = AssetsDescriptors::new();

        dbg!("{:?}", &config);

        let textures_desc = read_textures_descriptors(&config.textures_cfg)?;
        textures_desc.into_iter().for_each(|t| ad.push(t));

        let materials_desc = read_materials_descriptors(&config.materials_cfg)?;
        materials_desc.into_iter().for_each(|t| ad.push(t));

        let meshes_desc = read_mesh_descriptors(&config.meshes_cfg)?;
        meshes_desc.into_iter().for_each(|t| ad.push(t));

        let model_desc = read_models_descriptors(&config.models_cfg)?;
        model_desc.into_iter().for_each(|t| ad.push(t));

        dbg!("{:?}", &ad);

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
