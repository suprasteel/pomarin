use std::path::Path;
use std::rc::Rc;
use std::{fmt::Debug, ops::Deref, path::PathBuf};

use super::texture::{self, TextureKind};

#[derive(Clone, Copy, Hash, PartialEq, std::cmp::Eq, Debug)]
pub enum MaterialKind {
    Texture,
    Color,
}

impl ToString for MaterialKind {
    fn to_string(&self) -> String {
        match self {
            MaterialKind::Texture => "MaterialKind::Texture".to_string(),
            MaterialKind::Color => "MaterialKind::Color".to_string(),
        }
    }
}

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
    ) -> Result<Self> {
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

        Ok(ColorMaterial {
            kind: MaterialKind::Color,
            name: name.as_ref().to_string(),
            bind_group,
        })
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

#[derive(Debug)]
pub struct TextureMaterialBuilder {
    name: Option<String>,
    diffuse_texture: Option<(String, PathBuf)>,
    normal_texture: Option<(String, PathBuf)>,
}

impl TextureMaterialBuilder {
    pub fn name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn diffuse_texture<S: AsRef<str>, P: AsRef<Path>>(mut self, name: S, path: P) -> Self {
        self.diffuse_texture = Some((name.as_ref().to_string(), path.as_ref().into()));
        self
    }

    pub fn normal_texture<S: AsRef<str>, P: AsRef<Path>>(mut self, name: S, path: P) -> Self {
        self.diffuse_texture = Some((name.as_ref().to_string(), path.as_ref().into()));
        self
    }

    fn unwrap(&self) -> Result<(String, (String, PathBuf), (String, PathBuf))> {
        let name = self
            .name
            .as_ref()
            .ok_or_else(|| anyhow!("Material builder incomplete ({}).", "name".to_string()))?;
        let unwrap_texture_name =
            |param: Option<&(String, PathBuf)>, tk: TextureKind| -> Result<(String, PathBuf)> {
                param.map_or_else(
                    || {
                        Err(anyhow!(
                            "Material builder {} incomplete ({}).",
                            name,
                            tk.to_string()
                        ))
                    },
                    |(a, b)| Ok((a.clone(), b.clone())),
                )
            };
        let diffuse = unwrap_texture_name(self.diffuse_texture.as_ref(), TextureKind::Diffuse)?;
        let normal = unwrap_texture_name(self.normal_texture.as_ref(), TextureKind::Normal)?;
        Ok((name.to_string(), diffuse, normal))
    }

    pub fn _build(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<TextureMaterial> {
        let (name, diffuse, normal) = self.unwrap()?;

        let diffuse_texture = Rc::new(texture::Texture::load(device, queue, diffuse.0, true)?);
        let normal_texture = Rc::new(texture::Texture::load(device, queue, normal.1, true)?);

        TextureMaterial::new(device, name.clone(), &diffuse_texture, &normal_texture)
    }
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
    ) -> Result<Self> {
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

        Ok(TextureMaterial {
            kind: MaterialKind::Texture,
            name: name.as_ref().to_string(),
            bind_group,
        })
    }

    /// load material using its name
    /// return material, textures and textures' names
    /// align on load colors (or align load colors on this)
    pub fn load_from_mtl<P>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
        material_name: String,
    ) -> Result<((Self, texture::Texture, texture::Texture), String, String)>
    where
        P: AsRef<Path> + Debug,
    {
        let mtl = tobj::load_mtl(path.as_ref())?;
        for material in mtl.0 {
            if material.name == material_name.as_ref() {
                return TextureMaterial::load(
                    device,
                    queue,
                    material_name,
                    path.as_ref().parent().context("error unimplemented")?,
                    // .context(format!("failed to use path {} to load {}", material_name.as_ref().to_string()))?,
                    material.diffuse_texture.clone(),
                    material.normal_texture.clone(),
                )
                .map(|ok_result| (ok_result, material.diffuse_texture, material.normal_texture));
            }
        }
        Err(anyhow!(Error::MaterialNotFound {
            material: material_name.clone()
        }))
    }

    // returns Material, diffuse texture, normal texture
    pub fn load<P: AsRef<Path>, S: AsRef<str>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: S,
        folder: P,
        diffuse_filename: S,
        normal_filename: S,
    ) -> Result<(Self, texture::Texture, texture::Texture)> {
        let folder = folder.as_ref();
        let diffuse_path = folder.join(diffuse_filename.as_ref());
        let normal_path = folder.join(normal_filename.as_ref());

        let diffuse_texture = texture::Texture::load(device, queue, diffuse_path, false)?;
        let normal_texture = texture::Texture::load(device, queue, normal_path, true)?;

        let material = Self::new(device, name, &diffuse_texture, &normal_texture)?;

        Ok((material, diffuse_texture, normal_texture))
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
