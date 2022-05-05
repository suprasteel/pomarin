use anyhow::Result;
use image::GenericImageView;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use super::{error::TextureError, resources::NamedHandle, wgpu_state::WgpuResourceLoader};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Texture {
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P,
        is_normal_map: bool,
    ) -> Result<Self> {
        // use load & texture as target ?
        debug!(target: "load", "Loading texture from file {:?}", path.as_ref().to_str());
        let path_copy = path.as_ref().to_path_buf();
        let label = path_copy.to_str();

        let img = image::open(path)?;
        Self::from_image(device, queue, &img, label, is_normal_map)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if is_normal_map {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                wgpu::TextureFormat::Rgba8UnormSrgb
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        debug!("Creating depth texture");
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

// TODO: check both are used
#[derive(Deserialize, Serialize, Debug)]
pub struct TextureDescriptor {
    name: String,
    path: PathBuf,
    kind: TextureKind,
}

impl TextureDescriptor {
    pub fn _new_(name: String, path: PathBuf, kind: TextureKind) -> Self {
        Self { name, path, kind }
    }
}

impl WgpuResourceLoader for TextureDescriptor {
    type Output = Rc<Texture>;

    fn load(&self, wgpu_state: &super::wgpu_state::WgpuState) -> Result<Self::Output> {
        if wgpu_state.store.contains_texture(&self.name) {
            return Ok(wgpu_state
                .store
                .get_texture(&self.name)
                .expect("Impossible err 2"));
        }
        let is_normal_map = self.kind == TextureKind::Normal;
        let texture = Texture::load(
            &wgpu_state.device,
            &wgpu_state.queue,
            &self.path,
            is_normal_map,
        )?;
        let texture = Rc::new(texture);
        wgpu_state.store.add_texture(&self.name, texture.clone());
        Ok(texture.clone())
    }
}

impl Display for TextureDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let k = if self.kind == TextureKind::Diffuse {
            "diffuse"
        } else {
            "normal"
        };
        write!(
            f,
            "TextureDescriptor:\"{}\" of type {} from {:?}",
            self.name, k, self.path
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq, Deserialize, Serialize)]
pub enum TextureKind {
    Diffuse,
    Normal,
}

impl From<TextureKind> for String {
    fn from(tk: TextureKind) -> Self {
        match tk {
            TextureKind::Diffuse => "TextureKind::Diffuse".to_string(),
            TextureKind::Normal => "TextureKind::Normal".to_string(),
        }
    }
}

impl TryFrom<&str> for TextureKind {
    type Error = TextureError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "TextureKind::Diffuse" => Ok(TextureKind::Diffuse),
            "TextureKind::Normal" => Ok(TextureKind::Normal),
            input => Err(TextureError::DeserialisationError {
                type_to_deser: "TextureKind".to_string(),
                input: input.to_string(),
            }),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub struct TextureName(String);

impl From<&str> for TextureName {
    fn from(value: &str) -> Self {
        TextureName(value.to_string())
    }
}

impl Deref for TextureName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NamedHandle<TextureName> for TextureDescriptor {
    fn name(&self) -> TextureName {
        TextureName(self.name.to_string())
    }
}

impl std::fmt::Display for TextureName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Texture({})", self.0)
    }
}
