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
    fn update<T>(self, mut updater: T) -> Self
    where
        T: CameraUpdater,
    {
        updater.update(self)
    }
}

pub trait CameraUpdater {
    fn update(&mut self, camera_uniform: CameraUniform) -> CameraUniform;
}

pub struct CameraSystem<T>
where
    T: CameraUpdater,
{
    updater: T,
    uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<T> CameraSystem<T>
where
    T: CameraUpdater,
{
    pub fn init(device: &wgpu::Device, updater: T) -> (wgpu::BindGroupLayout, CameraSystem<T>) {
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
                updater,
                uniform,
                buffer,
                bind_group,
            },
        )
    }

    pub fn update(&mut self, queue: &wgpu::Queue, _dt: Duration) {
        self.uniform = self.updater.update(self.uniform);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }
}

pub struct InputState {
    pub speed: f32,
    pub up: f32,
    pub down: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            speed: 1.0,
            up: 0.0,
            down: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

pub struct ViewState {
    width: u32,
    height: u32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    position: [f32; 3],
    target: [f32; 3],
    up: [f32; 3],
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            width: 16 * 100,
            height: 9 * 100,
            fovy: 45.0,
            znear: 1.0,
            zfar: 1000.0,
            position: [100.0, 100.0, 50.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        }
    }
}

pub struct OrbitController {
    _input: InputState,
    view: ViewState,
}

impl Default for OrbitController {
    fn default() -> Self {
        Self {
            _input: Default::default(),
            view: Default::default(),
        }
    }
}

impl OrbitController {
    fn pos(&self) -> cgmath::Point3<f32> {
        let p = self.view.position;
        cgmath::Point3::new(p[0], p[1], p[2])
    }

    fn tar(&self) -> cgmath::Point3<f32> {
        let t = self.view.target;
        cgmath::Point3::new(t[0], t[1], t[2])
    }

    fn up(&self) -> cgmath::Vector3<f32> {
        let u = self.view.up;
        cgmath::Vector3::new(u[0], u[1], u[2])
    }

    fn aspect(&self) -> f32 {
        self.view.width as f32 / self.view.height as f32
    }

    pub fn uniform(&self) -> CameraUniform {
        let ViewState {
            znear, zfar, fovy, ..
        } = self.view;
        let fovy = cgmath::Deg(fovy);
        let vm = cgmath::Matrix4::look_at_rh(self.pos(), self.tar(), self.up());
        let pm = OPENGL_TO_WGPU_MATRIX * cgmath::perspective(fovy, self.aspect(), znear, zfar);

        CameraUniform {
            view_position: self.pos().to_homogeneous().into(),
            view_proj: (pm * vm).into(),
        }
    }

    pub fn _handle_event() {}
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
    );

impl CameraUpdater for OrbitController {
    fn update(&mut self, _camera_uniform: CameraUniform) -> CameraUniform {
        self.uniform()
    }
}
