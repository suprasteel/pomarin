use std::{fmt::Display, ops::Deref};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{
    geometry::{GeometryBuf, GeometryDescriptor, GeometryName, GeometryVertices},
    resources::NamedHandle,
    vertex::ModelVertex,
    wgpu_state::{WgpuResourceLoader, WgpuState},
};

/// # Wgpu named geometries buffers
#[derive(Debug)]
pub struct MeshBuf {
    pub name: String,
    pub geometries: Vec<GeometryBuf>,
}

/// Describe the kind of file/source is a mesh from
#[derive(Deserialize, Serialize, Debug)]
pub enum VerticesSource {
    Obj(String),
    // one day...
}

impl VerticesSource {
    fn load(&self) -> Result<Vec<GeometryVertices<ModelVertex>>> {
        match &self {
            VerticesSource::Obj(path) => {
                let (obj_models, _) = tobj::load_obj(
                    path,
                    &tobj::LoadOptions {
                        triangulate: true,
                        single_index: true,
                        ..Default::default()
                    },
                )?;
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

/// # Describe a mesh.
///
/// ## Example:
///
/// ```
/// MeshDescriptor {
///     name: "zodiac",
///     source: VerticesSource::Obj("file.obj")
///     geometries: vec![
///         GeometryDescriptor { name: "part_x" }
///         GeometryDescriptor { name: "part_y" }
///     ],
/// }
/// ```
///
#[derive(Deserialize, Serialize, Debug)]
pub struct MeshDescriptor {
    name: String,
    source: VerticesSource,
    geometries: Vec<GeometryDescriptor>,
}

impl MeshDescriptor {
    pub fn count_geometries(&self) -> usize {
        self.geometries.len()
    }

    pub fn geometries_names(&self) -> Vec<GeometryName> {
        self.geometries.iter().map(|g| g.name()).collect()
    }

    pub fn geometries(&self) -> &Vec<GeometryDescriptor> {
        &self.geometries
    }
}

impl WgpuResourceLoader for MeshDescriptor {
    type Output = MeshBuf;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        let geometries_vertices = self.source.load()?;

        let geometries = geometries_vertices
            .iter()
            .map(|gv| {
                if self.geometries_names().contains(&gv.name()) {
                    Ok(gv.to_geometry(&wgpu_state.device))
                } else {
                    Err(anyhow!("Expected geometry does not match file loaded"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MeshBuf {
            name: self.name.to_string(),
            geometries,
        })
    }
}

impl NamedHandle<MeshName> for MeshDescriptor {
    fn name(&self) -> MeshName {
        MeshName(self.name.clone())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct MeshName(String);

impl Display for MeshName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mesh({})", self.0)
    }
}

impl Deref for MeshName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
