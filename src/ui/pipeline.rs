use std::ops::Deref;

use super::material::MaterialKind;

#[derive(Debug)]
pub struct NamedPipeline {
    name: String,
    pipeline: wgpu::RenderPipeline,
    supported_material_kind: Vec<MaterialKind>,
}

impl NamedPipeline {
    pub fn new<S: AsRef<str>>(
        name: S,
        pipeline: wgpu::RenderPipeline,
        materials: Vec<MaterialKind>,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            pipeline,
            supported_material_kind: materials,
        }
    }

    pub fn can_use(&self, material_kind: MaterialKind) -> bool {
        self.supported_material_kind
            .iter()
            .find(|mk| *mk == &material_kind)
            .map_or_else(|| false, |_| true)
    }

    pub fn needs_material(&self) -> bool {
        !self.supported_material_kind.is_empty()
    }
}

impl Deref for NamedPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}
