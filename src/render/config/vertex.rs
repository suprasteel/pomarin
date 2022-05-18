use super::{geometry::GeometryVertices, WgpuResourceLoader};
use crate::render::{scene::vertex::ModelVertex, state::WgpuState};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Describe the kind of file/source is a mesh from
#[derive(Deserialize, Debug)]
pub enum VerticesSource {
    Obj(String),
    // one day...
}

impl WgpuResourceLoader for VerticesSource {
    type Output = Vec<GeometryVertices<ModelVertex>>;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        let directory = PathBuf::from(wgpu_state.settings.meshes_directory.to_string());
        match &self {
            VerticesSource::Obj(path) => {
                log::info!("Load obj {} ({:?})", path.to_string(), directory);
                let (obj_models, _) = tobj::load_obj(
                    directory.join(path),
                    &tobj::LoadOptions {
                        triangulate: true,
                        single_index: true,
                        ..Default::default()
                    },
                )
                .context(format!(
                    "Failed to load obj {} ({:?})",
                    path.to_string(),
                    directory
                ))?;
                Ok(obj_models
                    .into_iter()
                    .map(|tobj_model| {
                        let mut vertices = Vec::new();
                        ModelVertex::fill_vertices_from_model(&mut vertices, &tobj_model);
                        GeometryVertices::new(&tobj_model.name, vertices, tobj_model.mesh.indices)
                    })
                    .collect())
            }
        }
    }
}
