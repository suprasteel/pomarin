use super::geometry::GeometryBuf;

/// # Wgpu named geometries buffers
#[derive(Debug)]
pub struct MeshBuf {
    pub name: String,
    pub geometries: Vec<GeometryBuf>,
}
