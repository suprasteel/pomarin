use std::{fmt::Debug, ops::Deref};

use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use super::{
    error::MaterialError,
    resources::NamedHandle,
    texture::{self, TextureDescriptor},
};

#[derive(Clone, Copy, Hash, PartialEq, std::cmp::Eq, Debug)]
pub enum MaterialKind {
    Texture,
    Color,
}

impl From<MaterialKind> for String {
    fn from(mk: MaterialKind) -> Self {
        match mk {
            MaterialKind::Texture => "MaterialKind::Texture".to_string(),
            MaterialKind::Color => "MaterialKind::Color".to_string(),
        }
    }
}

impl TryFrom<&str> for MaterialKind {
    type Error = MaterialError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "MaterialKind::Texture" => Ok(MaterialKind::Texture),
            "MaterialKind::Color" => Ok(MaterialKind::Color),
            input => Err(MaterialError::DeserialisationError {
                type_to_deser: "MaterialKind".to_string(),
                input: input.to_string(),
            }),
        }
    }
}

/// In the end a material is just a bind group with atached data (as texture or uniform buffer)
pub trait Material: ToString + Debug + Deref<Target = wgpu::BindGroup> {
    fn name(&self) -> String;
    fn kind(&self) -> MaterialKind;
}

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
    pub name: String,
    pub bind_group: wgpu::BindGroup,
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
    fn new<S: AsRef<str>>(
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

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("material color uniform buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
            label: Some("material buffer bind group"),
        });

        ColorMaterial {
            kind: MaterialKind::Color,
            name: name.as_ref().to_string(),
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

impl ToString for ColorMaterial {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Debug)]
pub struct TextureMaterial {
    kind: MaterialKind,
    pub name: String,
    pub bind_group: wgpu::BindGroup,
}

impl Deref for TextureMaterial {
    type Target = wgpu::BindGroup;

    fn deref(&self) -> &Self::Target {
        &self.bind_group
    }
}

impl Material for TextureMaterial {
    fn kind(&self) -> MaterialKind {
        self.kind
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl TextureMaterial {
    // create a new textured material using loaded textures
    fn new<S: AsRef<str>>(
        device: &wgpu::Device,
        name: S,
        diffuse_texture: &texture::Texture,
        normal_texture: &texture::Texture,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(&name.as_ref().to_string()),
        });

        TextureMaterial {
            kind: MaterialKind::Texture,
            name: name.as_ref().to_string(),
            bind_group,
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture material bind group layout"),
        })
    }
}

impl ToString for TextureMaterial {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TextureMaterialDescriptor {
    name: String,
    diffuse_texture: TextureDescriptor,
    normal_texture: TextureDescriptor,
}

impl NamedHandle<MaterialName> for TextureMaterialDescriptor {
    fn named_handle(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ColorMaterialDescriptor {
    name: String,
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: f32,
}

impl NamedHandle<MaterialName> for ColorMaterialDescriptor {
    fn named_handle(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct MaterialName(String);

impl Deref for MaterialName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for MaterialName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Material({})", self.0)
    }
}

/* pub fn _build(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<TextureMaterial> {
let (name, diffuse, normal) = self.unwrap()?;

let diffuse_texture = Rc::new(Texture::load(device, queue, diffuse.0, true)?);
let normal_texture = Rc::new(Texture::load(device, queue, normal.1, true)?);

Ok(TextureMaterial::new(
device,
name.clone(),
&diffuse_texture,
&normal_texture,
))
}*/
