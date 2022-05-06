pub mod camera;
pub mod egui; // egui root render
pub mod error;
pub mod geometry;
pub mod instance;
pub mod light;
pub mod loader;
pub mod material;
pub mod mesh;
pub mod model;
pub mod object;
pub mod objects_pass; // 3d root render
pub mod pipeline;
pub mod rpass;
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
        fn set_emitter_from(&mut self, event_loop: &EventLoop<T>);
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

pub mod resources {

    pub trait NamedHandle<H> {
        fn name(&self) -> H;
    }
}

pub mod wgpu_state {

    use super::store::Store;
    use crate::settings::{assets::AssetsDescriptors, config::ResourcesConfig, utils::load_assets};
    use anyhow::Result;
    use winit::dpi::PhysicalSize;

    pub struct WgpuState {
        // app conf
        pub instance: wgpu::Instance,
        pub surface: wgpu::Surface,
        pub config: wgpu::SurfaceConfiguration,
        pub adapter: wgpu::Adapter,
        pub device: wgpu::Device,
        pub queue: wgpu::Queue,
        pub surface_format: wgpu::TextureFormat,
        pub assets: AssetsDescriptors,
        pub settings: ResourcesConfig,
        pub store: Store,
    }

    // retain wgpu state
    impl WgpuState {
        pub(crate) fn init(window: &winit::window::Window, settings: &ResourcesConfig) -> Self {
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

            let store = Store::new();
            let assets = load_assets(settings).expect("asset loading failure");

            Self {
                instance,
                surface,
                config,
                adapter,
                device,
                queue,
                surface_format,
                assets,
                settings: settings.to_owned(),
                store,
            }
        }

        pub(crate) fn pre_resize(&mut self, size: PhysicalSize<u32>) {
            if size.width > 0 && size.height > 0 {
                self.config.width = size.width;
                self.config.height = size.height;
            }
        }

        pub(crate) fn post_resize(&mut self, size: PhysicalSize<u32>) {
            if size.width > 0 && size.height > 0 {
                self.surface.configure(&self.device, &self.config)
            }
        }
    }

    pub trait WgpuResourceLoader {
        type Output;

        /// load resource if not already in store
        fn load(&self, wgpu_state: &WgpuState) -> Result<Self::Output>;
    }
}
