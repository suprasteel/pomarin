use crate::render::names::{MaterialName, MeshName, ModelName, NamedHandle, TextureName};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

use super::{
    material::MaterialDescriptor, mesh::MeshDescriptor, model::ModelDescriptor,
    texture::TextureDescriptor,
};

#[derive(Debug)]
pub enum AssetDescriptor {
    Texture(TextureDescriptor),
    Material(MaterialDescriptor),
    Mesh(MeshDescriptor),
    Model(ModelDescriptor),
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum AssetName {
    Texture(TextureName),
    Material(MaterialName),
    Mesh(MeshName),
    Model(ModelName),
}

/// Store assets descriptor to reuse on demand
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

    pub fn get<K: Into<AssetName>>(&self, key: K) -> Result<&AssetDescriptor> {
        let asset_name: &AssetName = &key.into();
        self.0.get(asset_name).map_or_else(
            || {
                Err(anyhow!(AssetError::AssetNotFound {
                    key: format!("{}", asset_name)
                }))
            },
            |descriptor| Ok(descriptor),
        )
    }
}

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Could not get reference of {target_type} from {descriptor}")]
    TryAsRefFailed {
        descriptor: AssetName,
        target_type: String,
    },
    #[error("{key} not found")]
    AssetNotFound { key: String },
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
            Err(anyhow!(AssetError::TryAsRefFailed {
                target_type: "ModelDescriptor".to_string(),
                descriptor: self.name(),
            }))
        }
    }
}

impl TryAsRef<MeshDescriptor> for AssetDescriptor {
    type Error = anyhow::Error;
    fn try_as_ref(&self) -> Result<&MeshDescriptor, Self::Error> {
        if let AssetDescriptor::Mesh(mesh) = self {
            Ok(mesh)
        } else {
            Err(anyhow!(AssetError::TryAsRefFailed {
                target_type: "MeshDescriptor".to_string(),
                descriptor: self.name(),
            }))
        }
    }
}

impl TryAsRef<TextureDescriptor> for AssetDescriptor {
    type Error = anyhow::Error;
    fn try_as_ref(&self) -> Result<&TextureDescriptor, Self::Error> {
        if let AssetDescriptor::Texture(t) = self {
            Ok(t)
        } else {
            Err(anyhow!(AssetError::TryAsRefFailed {
                target_type: "TextureDescriptor".to_string(),
                descriptor: self.name(),
            }))
        }
    }
}

impl TryAsRef<MaterialDescriptor> for AssetDescriptor {
    type Error = anyhow::Error;
    fn try_as_ref(&self) -> Result<&MaterialDescriptor, Self::Error> {
        if let AssetDescriptor::Material(m) = self {
            Ok(m)
        } else {
            Err(anyhow!(AssetError::TryAsRefFailed {
                target_type: "MaterialDescriptor".to_string(),
                descriptor: self.name(),
            }))
        }
    }
}

impl NamedHandle<AssetName> for AssetDescriptor {
    fn name(&self) -> AssetName {
        match self {
            AssetDescriptor::Texture(t) => AssetName::Texture(t.name()),
            AssetDescriptor::Material(m) => AssetName::Material(m.name()),
            AssetDescriptor::Mesh(m) => AssetName::Mesh(m.name()),
            AssetDescriptor::Model(m) => AssetName::Model(m.name()),
        }
    }
}

impl Display for AssetDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssetDesciptor({})", self.name().to_string())
    }
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
