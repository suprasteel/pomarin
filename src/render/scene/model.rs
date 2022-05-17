use std::rc::Rc;

use super::{material::Material, mesh::MeshBuf, pipeline::NamedPipeline};

/// A Wgpu-ready model
///
/// This struct points to the wgpu pipeline to use,
/// the mesh buffers to be used,
/// the materials (as bind groups) to apply to eac of the mesh's geometries
///
#[derive(Debug)]
pub struct Model {
    pub(crate) name: String,
    pub pipeline: Rc<NamedPipeline>,
    pub mesh: Rc<MeshBuf>,
    pub materials: Vec<Rc<dyn Material>>,
}

impl Model {
    pub fn new(name: String, pipeline: Rc<NamedPipeline>, mesh: Rc<MeshBuf>) -> Self {
        Self {
            name,
            pipeline,
            mesh,
            materials: vec![],
        }
    }
}

impl Ord for Model {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.pipeline.as_ref().name() != other.pipeline.as_ref().name() {
            self.pipeline
                .as_ref()
                .name()
                .cmp(&other.pipeline.as_ref().name())
        } else {
            self.mesh.name.cmp(&other.mesh.name)
        }
    }
}

impl Eq for Model {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for Model {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self
            .pipeline
            .as_ref()
            .name()
            .partial_cmp(&other.pipeline.as_ref().name())
        {
            Some(core::cmp::Ordering::Equal) => Some(self.mesh.name.cmp(&other.mesh.name)),
            ord => return ord,
        }
    }
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline.as_ref().name() == other.pipeline.as_ref().name()
            && self.mesh.name == other.mesh.name
            && self.materials.len() == other.materials.len()
            && self
                .materials
                .iter()
                .fold("".to_string(), |acc, mat| format!("{} {}", acc, mat.name()))
                == other
                    .materials
                    .iter()
                    .fold("".to_string(), |acc, mat| format!("{} {}", acc, mat.name()))
    }
}
