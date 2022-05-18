use std::ops::Deref;

use wgpu::util::DeviceExt;

use super::material::{Material, MaterialKind};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    ambient: [f32; 3],
    specular: f32,
    diffuse: [f32; 3],
    _pad: f32,
}

#[derive(Debug)]
pub struct ColorMaterial {
    kind: MaterialKind,
    name: String,
    bind_group: wgpu::BindGroup,
}

impl Deref for ColorMaterial {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.bind_group
    }
}

impl Material for ColorMaterial {
    fn kind(&self) -> MaterialKind {
        self.kind
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl ColorMaterial {
    pub(crate) fn new<S: AsRef<str>>(
        device: &wgpu::Device,
        name: S,
        ambient: [f32; 3],
        diffuse: [f32; 3],
        specular: [f32; 3],
    ) -> Self {
        let uniform = ColorUniform {
            ambient,
            specular: (specular[0] + specular[1] + specular[2]) / 3.0,
            diffuse,
            _pad: 0.0,
        };

        let name = name.as_ref().to_string();

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} material color uniform buffer", name).to_string()),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
            label: Some(&format!("{} material buffer bind group", name).to_string()),
        });

        ColorMaterial {
            kind: MaterialKind::Color,
            name,
            bind_group,
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("color material bind group layout"),
        })
    }
}
