use std::{fmt::Debug, ops::Deref, rc::Rc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use super::{
    error::MaterialError,
    resources::NamedHandle,
    texture::{self, TextureDescriptor},
    wgpu_state::{WgpuResourceLoader, WgpuState},
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
pub enum MaterialDescriptor {
    Texture(TextureMaterialDescriptor),
    Color(ColorMaterialDescriptor),
}

impl NamedHandle<MaterialName> for MaterialDescriptor {
    fn name(&self) -> MaterialName {
        match self {
            MaterialDescriptor::Color(color) => color.name(),
            MaterialDescriptor::Texture(texture) => texture.name(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TextureMaterialDescriptor {
    name: String,
    pub diffuse_texture: TextureDescriptor,
    pub normal_texture: TextureDescriptor,
}

//TODO: delete after made better
impl TextureMaterialDescriptor {
    pub fn _new_(
        name: String,
        diffuse_texture: TextureDescriptor,
        normal_texture: TextureDescriptor,
    ) -> Self {
        Self {
            name,
            diffuse_texture,
            normal_texture,
        }
    }
}

impl NamedHandle<MaterialName> for TextureMaterialDescriptor {
    fn name(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ColorMaterialDescriptor {
    name: String,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

impl ColorMaterialDescriptor {
    pub fn _new_(name: String, ambient: [f32; 3], diffuse: [f32; 3], specular: [f32; 3]) -> Self {
        Self {
            name,
            ambient,
            diffuse,
            specular,
        }
    }
}

impl NamedHandle<MaterialName> for ColorMaterialDescriptor {
    fn name(&self) -> MaterialName {
        MaterialName(self.name.clone())
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct MaterialName(String);

impl From<&str> for MaterialName {
    fn from(value: &str) -> Self {
        MaterialName(value.to_string())
    }
}

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

impl WgpuResourceLoader for MaterialDescriptor {
    type Output = Rc<dyn Material>;

    fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output> {
        if wgpu_state.store.contains_material(&self.name()) {
            return Ok(wgpu_state
                .store
                .get_material(&self.name().deref())
                .expect("Impossible err 4"));
        }
        let material: Rc<dyn Material> = match self {
            MaterialDescriptor::Color(color) => {
                let material = ColorMaterial::new(
                    &wgpu_state.device,
                    color.name().deref(),
                    color.ambient,
                    color.diffuse,
                    color.specular,
                );
                Rc::new(material)
            }
            MaterialDescriptor::Texture(texture) => {
                let diffuse = texture.diffuse_texture.load(wgpu_state)?;
                let normal = texture.normal_texture.load(wgpu_state)?;

                Rc::new(TextureMaterial::new(
                    &wgpu_state.device,
                    texture.name().deref(),
                    diffuse.as_ref(),
                    normal.as_ref(),
                ))
            }
        };
        wgpu_state.store.add_material(material.clone());
        Ok(material)
    }
}
