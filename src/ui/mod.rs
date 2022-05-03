pub mod camera;
pub mod egui;
pub mod error;
pub mod geometry;
pub mod instance;
pub mod light;
pub mod material;
pub mod mesh;
pub mod model;
pub mod object;
pub mod pipeline;
pub mod store;
pub mod texture;
pub mod vertex;

pub mod event {
    use std::sync::Mutex;

    use anyhow::Result;
    use winit::event_loop::EventLoop;

    #[derive(Debug, Clone, Copy)]
    pub enum PomarinEvent {
        SomeEvent,
        EguiRequestRedraw,
        CloseApp,
    }

    pub trait EventEmitter<T> {
        fn set_emitter_from(&mut self, proxy: &EventLoop<T>);
        fn emit(&self, event: T) -> Result<()>;
    }

    pub struct Emitter<T>(std::sync::Mutex<winit::event_loop::EventLoopProxy<T>>)
    where
        T: 'static + Sync + Send + std::fmt::Debug;

    impl<T> Emitter<T>
    where
        T: 'static + Sync + Send + std::fmt::Debug,
    {
        pub fn new(event_loop: &EventLoop<T>) -> Self {
            Self(Mutex::new(event_loop.create_proxy()))
        }

        pub fn emit(&self, event: T) -> Result<()> {
            Ok(self.0.lock().unwrap().send_event(event)?)
        }
    }

    impl epi::backend::RepaintSignal for Emitter<PomarinEvent> {
        fn request_repaint(&self) {
            self.0
                .lock()
                .unwrap()
                .send_event(PomarinEvent::EguiRequestRedraw)
                .ok();
        }
    }
}

pub mod wgpu_state {
    use winit::dpi::PhysicalSize;

    pub struct WgpuState {
        pub instance: wgpu::Instance,
        pub surface: wgpu::Surface,
        pub config: wgpu::SurfaceConfiguration,
        pub adapter: wgpu::Adapter,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub surface_format: wgpu::TextureFormat,
    }

    // retain wgpu state
    impl WgpuState {
        pub(crate) fn init(window: &winit::window::Window) -> Self {
            // should read config files and load models during init
            // let mut wgpu_context = pollster::block_on(Wgpu::new(&window));
            let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
            let surface = unsafe { instance.create_surface(&window) };
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }))
                .unwrap();

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            ))
            .unwrap();

            let size = window.inner_size();
            let surface_format = surface.get_preferred_format(&adapter).unwrap();
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Immediate,
            };
            surface.configure(&device, &config);

            Self {
                instance,
                surface,
                config,
                adapter,
                device,
                queue,
                surface_format,
            }
        }

        pub(crate) fn update_size(&mut self, size: PhysicalSize<u32>) {
            if size.width > 0 && size.height > 0 {
                self.config.width = size.width;
                self.config.height = size.height;
                self.surface.configure(&self.device, &self.config)
            }
        }
    }
}

pub mod utils {}
