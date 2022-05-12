use std::{fmt::Display, ops::Deref, path::PathBuf, rc::Rc};

use anyhow::{anyhow, Context, Result};
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

/// # Describe a mesh.
///
/// Used to describe mesh configuration.
/// This struct can be deserialized.
/// It implements WgpuResourceLoader to instanciate a MeshBuf using wgpu state and being usable to
/// render the mesh.
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
    ///to be deleted
    pub fn _new_(
        name: String,
        source: VerticesSource,
        geometries: Vec<GeometryDescriptor>,
    ) -> Self {
        Self {
            name,
            source,
            geometries,
        }
    }

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

/// Builds the MeshBuf using wgpu state
impl WgpuResourceLoader for MeshDescriptor {
    type Output = Rc<MeshBuf>;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        log::info!("load {}", self.name());

        if wgpu_state.store.contains_mesh(&self.name) {
            return Ok(wgpu_state.store.get_mesh(&self.name).unwrap());
        }

        let geometries_vertices = self.source.load(wgpu_state)?;

        let geometries = geometries_vertices
            .iter()
            .map(|gv| {
                if self.geometries_names().contains(&gv.name()) {
                    Ok(gv.to_wgpu_geometry_buffer(&wgpu_state.device))
                } else {
                    Err(anyhow!("Expected geometry does not match file loaded"))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mesh = Rc::new(MeshBuf {
            name: self.name.to_string(),
            geometries,
        });
        wgpu_state.store.add_mesh(mesh.clone());
        Ok(mesh)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Hash)]
pub struct MeshName(String);

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

/// Describe the kind of file/source is a mesh from
#[derive(Deserialize, Serialize, Debug)]
pub enum VerticesSource {
    Obj(String),
    // one day...
}

impl VerticesSource {
    fn _load(&self) -> Result<Vec<GeometryVertices<ModelVertex>>> {
        match &self {
            VerticesSource::Obj(path) => {
                let (obj_models, _) = tobj::load_obj(
                    path,
                    &tobj::LoadOptions {
                        triangulate: true,
                        single_index: true,
                        ..Default::default()
                    },
                )
                .context(format!("Failed to load obj file {}", path.to_string()))?;
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
