mod render_engine;

use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::App;
use render_engine::Wgpu;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder,
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

    let mut ui = Ui::new();
    ui.set_max_fps(60);
    let emitter = ui.get_emitter();

    thread::spawn(move || loop {
        emitter
            .send_event(PomarinEvent::SomeEvent)
            .err()
            .map(|e| log::error!("oops: {:?}", e));
        //log::info!("s");
        thread::sleep(Duration::new(1, 0));
    });

    ui.run();
}

struct Gui {
    name: String,
    age: u32,
    emitter: Option<Arc<AppSignal>>,
}

impl Gui {
    fn set_emitter(&mut self, e: Arc<AppSignal>) {
        self.emitter = Some(e);
    }
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            emitter: None,
        }
    }
}

impl epi::App for Gui {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui");
            if ui.button("Close").clicked() {
                self.emitter.as_ref().map(|s| s.close());
                // frame.quit();
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

struct AppSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<PomarinEvent>>);

impl epi::backend::RepaintSignal for AppSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(PomarinEvent::EguiRequestRedraw)
            .ok();
    }
}

impl AppSignal {
    fn close(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(PomarinEvent::CloseApp)
            .ok();
    }
    fn some_evt(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(PomarinEvent::SomeEvent)
            .ok();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PomarinEvent {
    SomeEvent,
    EguiRequestRedraw,
    CloseApp,
}

pub trait UserRpass {}

pub struct Ui {
    event_loop: EventLoop<PomarinEvent>,
    _max_fps: u64,
    fps_time: Duration,
    initial_size: PhysicalSize<u32>,
    rpass_initializers: Vec<Box<dyn FnOnce(Wgpu) -> Box<dyn UserRpass>>>,
    rpasses: Vec<Box<dyn UserRpass>>,
    gui: Gui,
}

impl Ui {
    pub fn new() -> Self {
        let max_fps = 120;
        let fps_time = Duration::from_millis(1_000 / max_fps);
        let event_loop = EventLoop::<PomarinEvent>::with_user_event();
        Self {
            event_loop,
            _max_fps: max_fps,
            fps_time,
            initial_size: PhysicalSize::new(200, 200),
            rpass_initializers: vec![],
            rpasses: vec![],
            gui: Gui::default(),
        }
    }

    pub fn set_initilizers<F>(&self, _fns: Vec<F>)
    where
        F: FnOnce(Wgpu) -> Box<dyn UserRpass>,
    {
    }

    pub fn set_max_fps(&mut self, max: u64) {
        self._max_fps = max;
    }

    pub fn get_emitter(&self) -> EventLoopProxy<PomarinEvent> {
        self.event_loop.create_proxy()
    }

    // duration and measure time
    fn elapsed_time_since_last_frame(last_frame_time: Instant) -> (Duration, Instant) {
        let now = std::time::Instant::now();
        (now - last_frame_time, now)
    }

    // cannot use ui after run
    pub fn run(mut self) {
        let window = WindowBuilder::new().build(&self.event_loop).unwrap();
        window.set_title(WINDOW_NAME);
        window.set_decorations(false);
        window.set_maximized(true);
        window.set_visible(true);
        window.set_min_inner_size(Some(self.initial_size));

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
        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &config);

        let signal = std::sync::Arc::new(AppSignal(std::sync::Mutex::new(
            self.event_loop.create_proxy(),
        )));

        self.gui.set_emitter(signal.clone());

        // We use the egui_winit_platform crate as the platform.
        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

        // init_rpass_xx(&context) -> UserRpass

        log::info!("Starting event loop");
        let mut previous_frame_time = None;
        let mut last_frame_time = std::time::Instant::now();

        self.event_loop.run(move |event, _, control_flow| {
            platform.handle_event(&event);
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
                                log::info!(target: "event", "Window closed on user request");
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(size) => {
                                let size = *size;
                                log::info!(target: "event", "Window resized to {:?}", size);
                                if size.width > 0 && size.height > 0 {
                                    config.width = size.width;
                                    config.height = size.height;
                                    surface.configure(&device, &config)
                                }
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
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let output_frame = match surface.get_current_texture() {
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

                    // Begin to draw the UI frame.
                    let egui_start = Instant::now();
                    platform.begin_frame();
                    let app_output = epi::backend::AppOutput::default();

                    let mut frame = epi::Frame::new(epi::backend::FrameData {
                        info: epi::IntegrationInfo {
                            name: "egui",
                            web_info: None,
                            cpu_usage: previous_frame_time,
                            native_pixels_per_point: Some(window.scale_factor() as _),
                            prefer_dark_mode: None,
                        },
                        output: app_output,
                        repaint_signal: signal.clone(),
                    });

                    // draw gui
                    self.gui.update(&platform.context(), &mut frame);

                    // End the UI frame. We could now handle the output and draw the UI with the backend.
                    let (_output, paint_commands) = platform.end_frame(Some(&window));
                    let paint_jobs = platform.context().tessellate(paint_commands);

                    let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
                    previous_frame_time = Some(frame_time);

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("encoder"),
                        });

                    // Upload all resources for the GPU.
                    let screen_descriptor = ScreenDescriptor {
                        physical_width: config.width,
                        physical_height: config.height,
                        scale_factor: window.scale_factor() as f32,
                    };
                    egui_rpass.update_texture(&device, &queue, &platform.context().font_image());
                    egui_rpass.update_user_textures(&device, &queue);
                    egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                    // Record all render passes.
                    egui_rpass
                        .execute(
                            &mut encoder,
                            &output_view,
                            &paint_jobs,
                            &screen_descriptor,
                            Some(wgpu::Color::BLACK),
                        )
                        .unwrap();
                    // Submit the commands.
                    queue.submit(std::iter::once(encoder.finish()));

                    // Redraw egui
                    output_frame.present();
                    let (dt, now) = Ui::elapsed_time_since_last_frame(last_frame_time);
                    last_frame_time = now;
                    /*if dt < self.fps_time {
                    thread::sleep(self.fps_time - dt);
                    }*/
                    // wgpu_context.render(dt).err().map(|e| log::error!("{}", e));
                }
                _ => {}
            };
        });
    }
}
