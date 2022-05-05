pub mod config;

pub mod assets {
    use std::{collections::HashMap, fmt::Display, fs, path::Path};

    use anyhow::{anyhow, Context, Result};

    use crate::ui::{
        material::{MaterialDescriptor, MaterialName},
        mesh::{MeshDescriptor, MeshName},
        model::{ModelDescriptor, ModelName},
        resources::NamedHandle,
        texture::{TextureDescriptor, TextureName},
    };

    use super::config::ResourcesConfig;

    #[derive(Debug)]
    pub enum AssetDescriptor {
        Texture(TextureDescriptor),
        Material(MaterialDescriptor),
        Mesh(MeshDescriptor),
        Model(ModelDescriptor),
    }
    impl Into<AssetDescriptor> for TextureDescriptor {
        fn into(self) -> AssetDescriptor {
            AssetDescriptor::Texture(self)
        }
    }
    impl Into<AssetDescriptor> for MaterialDescriptor {
        fn into(self) -> AssetDescriptor {
            AssetDescriptor::Material(self)
        }
    }
    impl Into<AssetDescriptor> for MeshDescriptor {
        fn into(self) -> AssetDescriptor {
            AssetDescriptor::Mesh(self)
        }
    }
    impl Into<AssetDescriptor> for ModelDescriptor {
        fn into(self) -> AssetDescriptor {
            AssetDescriptor::Model(self)
        }
    }

    pub trait TryAsRef<T> {
        type Error;
        fn try_as_ref(&self) -> Result<&T, Self::Error>;
    }

    impl TryAsRef<ModelDescriptor> for AssetDescriptor {
        type Error = anyhow::Error;
        fn try_as_ref(&self) -> Result<&ModelDescriptor, Self::Error> {
            if let AssetDescriptor::Model(model) = self {
                Ok(model)
            } else {
                Err(anyhow!("AssetDescriptor is not a ModelDescriptor"))
            }
        }
    }

    #[derive(Hash, Eq, PartialEq, Debug)]
    pub enum AssetName {
        Texture(TextureName),
        Material(MaterialName),
        Mesh(MeshName),
        Model(ModelName),
    }

    impl Display for AssetName {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AssetName::Texture(t) => write!(f, "{}", t),
                AssetName::Material(m) => write!(f, "{}", m),
                AssetName::Mesh(m) => write!(f, "{}", m),
                AssetName::Model(m) => write!(f, "{}", m),
            }
        }
    }

    impl Into<AssetName> for TextureName {
        fn into(self) -> AssetName {
            AssetName::Texture(self)
        }
    }

    impl Into<AssetName> for MaterialName {
        fn into(self) -> AssetName {
            AssetName::Material(self)
        }
    }
    impl Into<AssetName> for MeshName {
        fn into(self) -> AssetName {
            AssetName::Mesh(self)
        }
    }
    impl Into<AssetName> for ModelName {
        fn into(self) -> AssetName {
            AssetName::Model(self)
        }
    }
    #[derive(Debug)]
    pub struct AssetsDescriptors(HashMap<AssetName, AssetDescriptor>);

    impl AssetsDescriptors {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn push<D>(&mut self, descriptor: D)
        where
            D: Into<AssetDescriptor>,
        {
            let descriptor = descriptor.into();
            match descriptor {
                AssetDescriptor::Texture(ref t) => {
                    self.0.insert(AssetName::Texture(t.name()), descriptor);
                }
                AssetDescriptor::Material(ref mat) => {
                    self.0.insert(AssetName::Material(mat.name()), descriptor);
                }
                AssetDescriptor::Mesh(ref mesh) => {
                    self.0.insert(AssetName::Mesh(mesh.name()), descriptor);
                }
                AssetDescriptor::Model(ref model) => {
                    self.0.insert(AssetName::Model(model.name()), descriptor);
                }
            }
        }

        pub fn find<K: Into<AssetName>>(&self, key: K) -> Option<&AssetDescriptor> {
            self.0.get(&key.into())
        }
    }

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
        let list: Vec<MaterialDescriptor> = ron::from_str(&string_content)?;
        Ok(list)
    }
    pub fn read_mesh_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<MeshDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading meshes file")?;
        let list: Vec<MeshDescriptor> = ron::from_str(&string_content)?;
        Ok(list)
    }
    pub fn read_textures_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<TextureDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading textures file")?;
        let list: Vec<TextureDescriptor> = ron::from_str(&string_content)?;
        Ok(list)
    }
    pub fn read_models_descriptors<P: AsRef<Path>>(file: P) -> Result<Vec<ModelDescriptor>> {
        let string_content = fs::read_to_string(file).context("reading models file")?;
        let list: Vec<ModelDescriptor> = ron::from_str(&string_content)?;
        Ok(list)
    }
}
