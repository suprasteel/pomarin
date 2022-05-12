use std::marker::PhantomData;

use wgpu::BufferAddress;

pub trait RawInstanceTrait: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl RawInstanceTrait for InstanceRaw {}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct InstancesSystem<I>
where
    I: RawInstanceTrait,
{
    buffer: wgpu::Buffer,
    instances_count: u32,
    _phamtom: PhantomData<I>,
}

const MAX_INSTANCES: u64 = 10;

impl<I> InstancesSystem<I>
where
    I: RawInstanceTrait + std::fmt::Debug,
{
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances buffer"),
            size: MAX_INSTANCES * std::mem::size_of::<I>() as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instances_count = 0;

        Self {
            buffer,
            instances_count,
            _phamtom: PhantomData,
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn count(&self) -> u32 {
        self.instances_count
    }

    pub fn set_instances_raw(&mut self, mut instances: Vec<I>, queue: &wgpu::Queue) {
        instances.truncate(MAX_INSTANCES as usize);
        self.instances_count = instances.len() as u32;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&instances));
    }
}
