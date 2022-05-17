use std::{fmt::Debug, ops::Deref};

use anyhow::Result;

use crate::render::error::MaterialError;

#[derive(Clone, Copy, Hash, PartialEq, std::cmp::Eq, Debug)]
pub enum MaterialKind {
    Texture,
    Color,
}

impl From<MaterialKind> for String {
    fn from(mk: MaterialKind) -> Self {
        match mk {
            MaterialKind::Texture => "MaterialKind::Texture".to_string(),
            MaterialKind::Color => "MaterialKind::Color".to_string(),
        }
    }
}

impl TryFrom<&str> for MaterialKind {
    type Error = MaterialError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "MaterialKind::Texture" => Ok(MaterialKind::Texture),
            "MaterialKind::Color" => Ok(MaterialKind::Color),
            input => Err(MaterialError::DeserialisationError {
                type_to_deser: "MaterialKind".to_string(),
                input: input.to_string(),
            }),
        }
    }
}

/// In the end a material is just a bind group
pub trait Material: Debug + Deref<Target = wgpu::BindGroup> {
    fn name(&self) -> String;
    fn kind(&self) -> MaterialKind;
}
