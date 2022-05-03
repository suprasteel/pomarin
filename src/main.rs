mod render_engine;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const WINDOW_NAME: &'static str = "Pomarin";

// used to specify logs scope
#[cfg(debug_assertions)]
const ENV_FILE: &str = "dev.env";

fn main() {
    println!(
        " --- Starting Pomarin Application (loading environment from {}) --- ",
        ENV_FILE
    );
    dotenv::from_filename(ENV_FILE).ok();
    env_logger::init();
    log::info!("Initialized environment and logger");

    let mut ui = AppUi::new();
    ui.set_max_fps(60);
    let emitter = ui.get_emitter();

    thread::spawn(move || loop {
        emitter
            .emit(PomarinEvent::SomeEvent)
            .err()
            .map(|e| log::error!("oops: {:?}", e));
        //log::info!("s");
        thread::sleep(Duration::new(1, 0));
    });

    ui.run();
}

struct EguiRoutine {
    name: String,
    age: u32,
    emitter: Option<Arc<Emitter>>,
}

impl Default for EguiRoutine {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            emitter: None,
        }
    }
}

trait EventEmitter<T> {
    fn set_emitter_from(&mut self, proxy: &EventLoop<T>);
    fn emit(&self, event: T) -> Result<()>;
}

impl EguiRoutine {
    fn close_app(&self) -> Result<()> {
        self.emit(PomarinEvent::CloseApp)
    }
}

impl EventEmitter<PomarinEvent> for EguiRoutine {
    fn emit(&self, event: PomarinEvent) -> Result<()> {
        self.emitter.as_ref().map_or_else(
            || Err(anyhow!("No emitter set for EguiRender")),
            |e| e.emit(event),
        )
    }

    fn set_emitter_from(&mut self, event_loop: &EventLoop<PomarinEvent>) {
        self.emitter = Some(Arc::new(Emitter::new(event_loop)));
    }
}

impl epi::App for EguiRoutine {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::Area::new("test")
            .fixed_pos(egui::pos2(10.0, 10.0))
            .show(ctx, |ui| {
                if ui.button("Close").clicked() {
                    self.close_app().err().map(|e| log::error!("{:?}", e));
                    _frame.quit();
                }
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    self.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
    }

    fn name(&self) -> &str {
        "test"
    }
}

pub struct Emitter(std::sync::Mutex<winit::event_loop::EventLoopProxy<PomarinEvent>>);

impl epi::backend::RepaintSignal for Emitter {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(PomarinEvent::EguiRequestRedraw)
            .ok();
    }
}

impl Emitter {
    fn new(event_loop: &EventLoop<PomarinEvent>) -> Self {
        Self(Mutex::new(event_loop.create_proxy()))
    }

    fn emit(&self, event: PomarinEvent) -> Result<()> {
        Ok(self.0.lock().unwrap().send_event(event)?)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PomarinEvent {
    SomeEvent,
    EguiRequestRedraw,
    CloseApp,
}

pub trait WgpuRPassBuilder {
    type Output: WgpuRpass;
    fn build(
        self,
        wgpu: &WgpuState,
        window: &Window,
        event_loop: &EventLoop<PomarinEvent>,
    ) -> Self::Output;
}

pub trait WgpuRpass {
    fn handle_event(&mut self, event: &Event<PomarinEvent>);
    fn render(
        &mut self,
        wgpu: &WgpuState,
        window: &winit::window::Window,
        output_view: &wgpu::TextureView,
        encoder: wgpu::CommandEncoder,
    ) -> wgpu::CommandEncoder;
}

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
    fn init(window: &winit::window::Window) -> Self {
        // should read config files and load models during init
        // let mut wgpu_context = pollster::block_on(Wgpu::new(&window));
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
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

    fn update_size(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config)
        }
    }
}

struct EguiWgpuPassBuilder<T> {
    gui: T,
}

impl<T> EguiWgpuPassBuilder<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    fn new(gui: T) -> Self {
        Self { gui }
    }
}

impl<T> WgpuRPassBuilder for EguiWgpuPassBuilder<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    type Output = EguiWgpuPass<T>;
    fn build(
        mut self,
        wgpu: &WgpuState,
        window: &Window,
        event_loop: &EventLoop<PomarinEvent>,
    ) -> Self::Output {
        let repainter = Arc::new(Emitter::new(event_loop));

        let inner_size = window.inner_size();
        let platform = Platform::new(PlatformDescriptor {
            physical_width: inner_size.width,
            physical_height: inner_size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let rpass = RenderPass::new(&wgpu.device, wgpu.surface_format, 1);

        self.gui.set_emitter_from(event_loop);

        Self::Output {
            platform,
            rpass,
            previous_frame_time: None,
            repainter,
            gui: self.gui,
        }
    }
}

// retain egui state
struct EguiWgpuPass<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    platform: Platform,
    rpass: egui_wgpu_backend::RenderPass,
    previous_frame_time: Option<f32>,
    repainter: Arc<dyn epi::backend::RepaintSignal>,
    gui: T,
}

impl<T> WgpuRpass for EguiWgpuPass<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    fn handle_event(&mut self, event: &Event<PomarinEvent>) {
        self.platform.handle_event(event);
    }

    fn render(
        &mut self,
        wgpu: &WgpuState,
        window: &winit::window::Window,
        output_view: &wgpu::TextureView,
        mut encoder: wgpu::CommandEncoder,
    ) -> wgpu::CommandEncoder {
        let egui_start = Instant::now();
        self.platform.begin_frame();
        let app_output = epi::backend::AppOutput::default();

        let mut frame = epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "egui",
                web_info: None,
                cpu_usage: self.previous_frame_time,
                native_pixels_per_point: Some(window.scale_factor() as _),
                prefer_dark_mode: None,
            },
            output: app_output,
            repaint_signal: self.repainter.clone(),
        });

        // draw gui
        self.gui.update(&self.platform.context(), &mut frame);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let (_output, paint_commands) = self.platform.end_frame(Some(&window));
        let paint_jobs = self.platform.context().tessellate(paint_commands);

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: wgpu.config.width,
            physical_height: wgpu.config.height,
            scale_factor: window.scale_factor() as f32,
        };
        self.rpass.update_texture(
            &wgpu.device,
            &wgpu.queue,
            &self.platform.context().font_image(),
        );
        self.rpass.update_user_textures(&wgpu.device, &wgpu.queue);
        self.rpass
            .update_buffers(&wgpu.device, &wgpu.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        encoder
    }
}

pub struct AppUi {
    event_loop: EventLoop<PomarinEvent>,
    _max_fps: u64,
    _fps_time: Duration,
    initial_size: PhysicalSize<u32>,
}

impl AppUi {
    pub fn new() -> Self {
        let max_fps = 120;
        let _fps_time = Duration::from_millis(1_000 / max_fps);
        let event_loop = EventLoop::<PomarinEvent>::with_user_event();
        Self {
            event_loop,
            _max_fps: max_fps,
            _fps_time,
            initial_size: PhysicalSize::new(200, 200),
        }
    }

    pub fn set_max_fps(&mut self, max: u64) {
        self._max_fps = max;
    }

    pub fn get_emitter(&self) -> Arc<Emitter> {
        Arc::new(Emitter::new(&self.event_loop))
    }

    // cannot use ui after run
    pub fn run(self) {
        let window = WindowBuilder::new().build(&self.event_loop).unwrap();
        window.set_title(WINDOW_NAME);
        window.set_decorations(false);
        window.set_maximized(true);
        window.set_visible(true);
        window.set_min_inner_size(Some(self.initial_size));

        let mut wgpu = WgpuState::init(&window);
        let mut egui = EguiWgpuPassBuilder::new(EguiRoutine::default()).build(
            &wgpu,
            &window,
            &self.event_loop,
        );
        // let mut rend = ObjectsPass::new(&wgpu, &window, &self.event_loop);

        log::info!("Starting event loop");

        self.event_loop.run(move |event, _, control_flow| {
            egui.handle_event(&event);
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } => {
                    if window_id == window.id() {
                        // wgpu_context.handle_window_event(event);
                        match event {
                            WindowEvent::CloseRequested => {
                                log::info!(target: "event", "Window CloseRequest");
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(size) => {
                                let size = *size;
                                wgpu.update_size(size);
                                log::info!(target: "event", "Window resized to {:?}", size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::UserEvent(event) => match event {
                    PomarinEvent::SomeEvent => {
                        log::info!(target: "event", "some user event");
                    }
                    PomarinEvent::EguiRequestRedraw => {
                        //egui
                        window.request_redraw();
                    }
                    PomarinEvent::CloseApp => {
                        log::info!(target: "event", "App close requested");
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let output_frame = match wgpu.surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(wgpu::SurfaceError::Outdated) => {
                            // This error occurs when the app is minimized on Windows.
                            // Silently return here to prevent spamming the console with:
                            // "The underlying surface has changed, and therefore the swap chain must be updated"
                            return;
                        }
                        Err(e) => {
                            eprintln!("Dropped frame with error: {}", e);
                            return;
                        }
                    };
                    let output_view = output_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let encoder =
                        wgpu.device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("encoder"),
                            });

                    let encoder = egui.render(&wgpu, &window, &output_view, encoder);
                    wgpu.queue.submit(std::iter::once(encoder.finish()));
                    // objects_pass.render()

                    // Redraw
                    output_frame.present();
                }
                _ => {}
            };
        });
    }
}
