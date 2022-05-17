use std::ops::Deref;

use super::{
    color_material::ColorMaterial, instance::InstanceRaw, material::MaterialKind, texture,
    texture_material::TextureMaterial, vertex::ModelVertex,
};

#[derive(Debug)]
pub struct NamedPipeline {
    name: String,
    pipeline: wgpu::RenderPipeline,
    supported_material_kind: Vec<MaterialKind>,
}

// TODO: PipelineName

impl NamedPipeline {
    pub fn new<S: AsRef<str>>(
        name: S,
        pipeline: wgpu::RenderPipeline,
        materials: Vec<MaterialKind>,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            pipeline,
            supported_material_kind: materials,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    // TODO; to be deleted after having a good way to check pipeline compat
    pub fn can_use(&self, material_kind: MaterialKind) -> bool {
        self.supported_material_kind
            .iter()
            .find(|mk| *mk == &material_kind)
            .map_or_else(|| false, |_| true)
    }

    pub fn needs_material(&self) -> bool {
        !self.supported_material_kind.is_empty()
    }
}

impl Deref for NamedPipeline {
    type Target = wgpu::RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

pub fn create_render_pipeline<S: ToString>(
    name: S,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&name.to_string()),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::OVER,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_textured_model_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    camera_bgl: &wgpu::BindGroupLayout,
    light_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let textures_bgl = TextureMaterial::bind_group_layout(device);
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Textured model render pipeline layout"),
        bind_group_layouts: &[&camera_bgl, &light_bgl, &textures_bgl],
        push_constant_ranges: &[],
    });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Textured model shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/texture.wgsl").into()),
    };
    create_render_pipeline(
        "Textured render pipeline",
        &device,
        &render_pipeline_layout,
        config.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[ModelVertex::desc(), InstanceRaw::desc()],
        shader,
    )
}

pub fn create_colored_model_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    camera_bgl: &wgpu::BindGroupLayout,
    light_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let colors_bgl = ColorMaterial::bind_group_layout(device);
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Colored model render pipeline layout"),
        bind_group_layouts: &[&camera_bgl, &light_bgl, &colors_bgl],
        push_constant_ranges: &[],
    });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Colored model shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/color.wgsl").into()),
    };
    create_render_pipeline(
        "Colored render pipeline",
        &device,
        &render_pipeline_layout,
        config.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[ModelVertex::desc(), InstanceRaw::desc()],
        shader,
    )
}
pub fn create_light_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    camera_bgl: &wgpu::BindGroupLayout,
    light_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Light Pipeline Layout"),
        bind_group_layouts: &[&camera_bgl, &light_bgl],
        push_constant_ranges: &[],
    });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Light Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light.wgsl").into()),
    };
    create_render_pipeline(
        "Light render pipeline",
        &device,
        &layout,
        config.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[ModelVertex::desc()],
        shader,
    )
}
