//! Represents the name of a resource wrapped in a type to prevent matching names from differents types

use super::{
    geometry::{GeometryDescriptor, GeometryVertices},
    material::{ColorMaterialDescriptor, MaterialDescriptor, TextureMaterialDescriptor},
    mesh::MeshDescriptor,
    model::ModelDescriptor,
    texture::TextureDescriptor,
};
use crate::ui::{
    names::NamedHandle,
    render_view::{geometry::GeometryBuf, model::Model},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct GeometryName(String);

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub struct ModelName(String);

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub struct TextureName(String);

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub struct MaterialName(String);

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Hash)]
pub struct MeshName(String);

// Geometry

impl From<&str> for GeometryName {
    fn from(s: &str) -> Self {
        GeometryName(s.to_string())
    }
}

impl Deref for GeometryName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for GeometryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Geometry({})", self.0)
    }
}

impl NamedHandle<GeometryName> for GeometryDescriptor {
    fn name(&self) -> GeometryName {
        GeometryName(self.name.clone())
    }
}

impl<T> NamedHandle<GeometryName> for GeometryVertices<T>
where
    T: bytemuck::Pod,
{
    fn name(&self) -> GeometryName {
        GeometryName(self.name.clone())
    }
}

impl NamedHandle<GeometryName> for GeometryBuf {
    fn name(&self) -> GeometryName {
        GeometryName(self.name.clone())
    }
}

//Model

impl From<&str> for ModelName {
    fn from(value: &str) -> Self {
        ModelName(value.to_string())
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model({})", self.0)
    }
}

impl Deref for ModelName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NamedHandle<ModelName> for ModelDescriptor {
    fn name(&self) -> ModelName {
        ModelName(self.name.to_string())
    }
}

impl NamedHandle<ModelName> for Model {
    fn name(&self) -> ModelName {
        ModelName(self.name.to_string())
    }
}

// Texture

impl From<&str> for TextureName {
    fn from(value: &str) -> Self {
        TextureName(value.to_string())
    }
}

impl Deref for TextureName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NamedHandle<TextureName> for TextureDescriptor {
    fn name(&self) -> TextureName {
        TextureName(self.name.to_string())
    }
}

// Materials

impl From<&str> for MaterialName {
    fn from(value: &str) -> Self {
        MaterialName(value.to_string())
    }
}

impl Deref for MaterialName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for MaterialName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Material({})", self.0)
    }
}

impl NamedHandle<MaterialName> for MaterialDescriptor {
    fn name(&self) -> MaterialName {
        match self {
            MaterialDescriptor::Color(color) => color.name(),
            MaterialDescriptor::Texture(texture) => texture.name(),
        }
    }
}

impl NamedHandle<MaterialName> for TextureMaterialDescriptor {
    fn name(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

impl NamedHandle<MaterialName> for ColorMaterialDescriptor {
    fn name(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

// Mesh
impl Display for MeshName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mesh({})", self.0)
    }
}

impl From<&str> for MeshName {
    fn from(value: &str) -> Self {
        MeshName(value.to_string())
    }
}

impl AsRef<str> for MeshName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for MeshName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NamedHandle<MeshName> for MeshDescriptor {
    fn name(&self) -> MeshName {
        MeshName(self.name.clone())
    }
}

// Texture

impl std::fmt::Display for TextureName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Texture({})", self.0)
    }
}
