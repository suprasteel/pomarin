use std::{fmt::Debug, ops::Deref};

#[derive(Clone, Copy, Hash, PartialEq, std::cmp::Eq, Debug)]
pub enum MaterialKind {
    Texture,
    Color,
}

impl ToString for MaterialKind {
    fn to_string(&self) -> String {
        match self {
            MaterialKind::Texture => "MaterialKind::Texture".to_string(),
            MaterialKind::Color => "MaterialKind::Color".to_string(),
        }
    }
}

pub trait Material: ToString + Debug + Deref<Target = wgpu::BindGroup> {
    fn name(&self) -> String;
    // fn bind_group(&self) -> &wgpu::BindGroup;
    fn kind(&self) -> MaterialKind;
}
