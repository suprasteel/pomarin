use std::time::Duration;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        let fov: f32 = 45.0;
        let far: f32 = 1000.0;
        let near: f32 = 1.0;
        let aspect: f32 = 16.0 / 9.0;
        Self {
            view_position: [0.0; 4],
            #[rustfmt::skip]
            view_proj: [
                [1.0/((fov/2.0).tan()*aspect), 0.0                , 0.0           , 0.0                  ],
                [0.0                         , 1.0/(fov/2.0).tan(), 0.0           , 0.0                  ],
                [0.0                         , 0.0                , far/(far-near), (far*near)/(near-far)],
                [0.0                         , 0.0                , 1.0           , 0.0                  ],
            ],
        }
    }
}

impl CameraUniform {
    fn update<F>(&mut self, f: F)
    where
        F: Fn(&mut CameraUniform),
    {
        f(self);
    }
}

pub struct CameraSystem<F> {
    updater: Option<F>,
    uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<F> CameraSystem<F>
where
    F: Fn(&mut CameraUniform),
{
    pub fn init(device: &wgpu::Device) -> (wgpu::BindGroupLayout, CameraSystem<F>) {
        let uniform = CameraUniform::default();

        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera.buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
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
            label: Some("camera.bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera.bind_group"),
        });

        (
            bind_group_layout,
            Self {
                updater: None,
                uniform,
                buffer,
                bind_group,
            },
        )
    }

    pub fn set_updater(&mut self, f: F) {
        self.updater = Some(f);
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        self.updater.as_ref().map(|f| self.uniform.update(f));
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }
}
