use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::resources::NamedHandle;

/// # Grouped buffers for indexed vertices of a geometry (simple mesh raw buffers)
///
/// - *name*: a convenient value to identify this geometry's buffers.
/// - *vertex_buffer*: wgpu::Buffer of vertices
/// - *index_buffer*: wgpu::Buffer of indexes to make faces
/// - *num_elements*: vertices count
///.
///
#[derive(Debug)]
pub struct Geometry {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl NamedHandle<GeometryName> for GeometryDescriptor {
    fn named_handle(&self) -> GeometryName {
        GeometryName(self.name.clone())
    }
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
}

/// Typed string to use as geometry name
#[derive(Deserialize, Serialize, Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct GeometryName(String);

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
