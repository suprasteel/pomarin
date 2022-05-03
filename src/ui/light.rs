use std::time::Duration;

use cgmath::prelude::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

impl LightUniformTrait for LightUniform {
    fn on_update(self, dt: Duration) -> Self {
        let position: cgmath::Vector3<_> = self.position.into();
        Self {
            position: (cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(60.0 * dt.as_secs_f32()),
            ) * position)
                .into(),
            _padding: 0,
            color: self.color,
            _padding2: 0,
        }
    }
}

/// has to be #[repr(C)]
pub trait LightUniformTrait: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn on_update(self, dt: Duration) -> Self;
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [1.0, 0.0, 0.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        }
    }
}

// TODO: Sun light system
pub struct LightSystem<U: LightUniformTrait> {
    light_uniform: U,
    light_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<U> LightSystem<U>
where
    U: LightUniformTrait,
{
    pub fn init(light_uniform: U, device: &wgpu::Device) -> (wgpu::BindGroupLayout, Self) {
        use wgpu::util::DeviceExt;
        // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light uniform buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        (
            bind_group_layout,
            Self {
                light_uniform,
                light_buffer,
                bind_group,
            },
        )
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        self.light_uniform = self.light_uniform.on_update(dt);

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}
