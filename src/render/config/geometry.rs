use crate::render::scene::geometry::GeometryBuf;
use serde::Deserialize;
use wgpu::util::DeviceExt;

/// Describe a geometry by its name.
///
/// # Example:
///
/// ```
/// GeometryDescriptor {
///     name: "part_x"
/// }
/// ```
///
/// Note: This struct should be used to
/// handle more parameters (options) or sanity checks
/// like the vertex count or the kind of vertex it
/// is made of (with or without uv, normals...)
#[derive(Deserialize, Debug)]
pub struct GeometryDescriptor {
    pub(crate) name: String,
}

impl From<&str> for GeometryDescriptor {
    fn from(value: &str) -> Self {
        GeometryDescriptor {
            name: value.to_string(),
        }
    }
}

/// Geometry vertices and indices
///
/// Container of the raw vertices the geometry is made of.
///
/// Is able to build a GeometryBuf from itself with the method to_wgpu_geometry_buffer(&device)
///.
pub struct GeometryVertices<T>
where
    T: bytemuck::Pod,
{
    pub(crate) name: String,
    pub(crate) vertices: Vec<T>,
    pub(crate) indices: Vec<u32>,
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

    pub fn to_wgpu_geometry_buffer(&self, device: &wgpu::Device) -> GeometryBuf {
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
