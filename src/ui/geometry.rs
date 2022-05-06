use std::ops::Deref;

use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use super::resources::NamedHandle;

/// # Grouped buffers for indexed vertices of a geometry (simple mesh raw buffers)
///
/// - *name*: a convenient value to identify this geometry's buffers. Only accessible from name()
/// function as a typed GeometryName.
/// - *vertex_buffer*: wgpu::Buffer of vertices
/// - *index_buffer*: wgpu::Buffer of indexes to make faces
/// - *num_elements*: vertices count
///.
#[derive(Debug)]
pub struct GeometryBuf {
    name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

/// # Describe a geometry by its name.
///
/// ## Example:
///
/// ```
/// GeometryDescriptor {
///     name: "part_x"
/// }
/// ```
#[derive(Deserialize, Serialize, Debug)]
pub struct GeometryDescriptor {
    name: String,
    // options ?
    // vertex count ? to check file ok ?
    // vertex type ?
}

impl From<&str> for GeometryDescriptor {
    fn from(value: &str) -> Self {
        GeometryDescriptor {
            name: value.to_string(),
        }
    }
}

/// # Geometry vertices and indices
///
/// The container of the raw vertices the geometry is made of
///.
pub struct GeometryVertices<T>
where
    T: bytemuck::Pod,
{
    name: String,
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
}

impl<T> GeometryVertices<T>
where
    T: bytemuck::Pod,
{
    pub fn new(name: &str, vertices: Vec<T>, indices: Vec<u32>) -> Self {
        GeometryVertices {
            name: name.to_string(),
            vertices,
            indices,
        }
    }

    pub fn to_geometry(&self, device: &wgpu::Device) -> GeometryBuf {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", self.name.to_string())),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", self.name.to_string())),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        GeometryBuf {
            name: self.name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
        }
    }
}

/// Typed string to use as geometry name
#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct GeometryName(String);

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

impl NamedHandle<GeometryName> for GeometryBuf {
    fn name(&self) -> GeometryName {
        GeometryName(self.name.clone())
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
