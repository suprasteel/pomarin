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
    pub(crate) name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}
