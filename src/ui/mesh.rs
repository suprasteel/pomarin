use std::{fmt::Display, ops::Deref, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use super::{
    geometry::{Geometry, GeometryDescriptor, GeometryName},
    resources::NamedHandle,
    vertex::ModelVertex,
};

/// A mesh carries geometries
#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub geometries: Vec<Geometry>,
}

impl Mesh {
    pub fn load_from_obj<P: AsRef<Path>, S: AsRef<str>>(
        device: &wgpu::Device,
        path: P,
        name: S,
    ) -> Result<Mesh> {
        let (obj_models, _) = tobj::load_obj(
            path.as_ref(),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;
        let mut meshes = Vec::new();
        obj_models.iter().for_each(|mesh| {
            let mut vertices = Vec::new();
            ModelVertex::fill_vertices_from_model(&mut vertices, &mesh);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&mesh.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            meshes.push(Geometry {
                name: mesh.name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: mesh.mesh.indices.len() as u32,
            });
        });

        Result::Ok(Mesh {
            name: name.as_ref().to_string(),
            geometries: meshes,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum MeshSource {
    Obj(String),
}

/// # Describe a mesh.
///
/// ## Example:
///
/// ```
/// MeshDescriptor {
///     name: "zodiac",
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
    file_name: MeshSource,
    geometries: Vec<GeometryDescriptor>,
}

impl MeshDescriptor {
    pub fn count_geometries(&self) -> usize {
        self.geometries.len()
    }

    pub fn geometries_names(&self) -> Vec<GeometryName> {
        self.geometries.iter().map(|g| g.named_handle()).collect()
    }

    pub fn geometries(&self) -> &Vec<GeometryDescriptor> {
        &self.geometries
    }
}

/* impl WgpuResourceLoader for MeshDescriptor {
    type Output = Mesh;

    fn load(&self, wgpu_state: &super::wgpu_state::WgpuState) -> Result<Self::Output> {

        let (obj_models, _) = tobj::load_obj(
            path.as_ref(),
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;
        let mut meshes = Vec::new();
        obj_models.iter().for_each(|mesh| {
            let mut vertices = Vec::new();
            ModelVertex::fill_vertices_from_model(&mut vertices, &mesh);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", path.as_ref())),
                contents: bytemuck::cast_slice(&mesh.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            meshes.push(Geometry {
                name: mesh.name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: mesh.mesh.indices.len() as u32,
            });
        });

        Result::Ok(Mesh {
            name: name.as_ref().to_string(),
            geometries: meshes,
        })
    }
}*/

impl NamedHandle<MeshName> for MeshDescriptor {
    fn named_handle(&self) -> MeshName {
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
