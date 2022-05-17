use anyhow::{anyhow, Result};
use std::rc::Rc;

use serde::Deserialize;

use crate::render::{names::NamedHandle, scene::mesh::MeshBuf, state::WgpuState};

use super::{
    geometry::GeometryDescriptor, handles::GeometryName, vertex::VerticesSource, WgpuResourceLoader,
};

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
#[derive(Deserialize, Debug)]
pub struct MeshDescriptor {
    pub(crate) name: String,
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
