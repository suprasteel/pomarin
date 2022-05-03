use std::path::Path;

use anyhow::Result;
use wgpu::util::DeviceExt;

use super::{geometry::Geometry, vertex::ModelVertex};

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
