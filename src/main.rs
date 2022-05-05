mod settings;
mod ui;

use std::{sync::Arc, thread, time::Duration};

use settings::config::AppConfig;
use ui::{event::Emitter, wgpu_state::WgpuState};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    settings::config::load_conf,
    ui::{
        egui::{EguiRoutine, EguiWgpuPassBuilder},
        event::PomarinEvent,
        loader::example_model,
        objects_pass::ObjectsPass,
    },
};

const APP_NAME: &'static str = "Pomarin";

// used to specify logs scope
const ENV_FILE: &str = "dev.env";

fn main() {
    println!(
        " --- Starting {} Application (loading environment from {}) --- ",
        APP_NAME, ENV_FILE
    );

    dotenv::from_filename(ENV_FILE).ok();

    env_logger::init();
    log::info!("Initialized environment and logger");

    example_model();

    let ui = AppUi::new(load_conf());
    let emitter = ui.get_emitter();

    thread::spawn(move || loop {
        emitter
            .emit(PomarinEvent::SomeEvent)
            .err()
            .map(|e| log::error!("oops: {:?}", e));
        thread::sleep(Duration::new(1, 0));
    });

    ui.run();
}

// reminder (may be removed)
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

pub struct AppUi {
    app_config: AppConfig,
    event_loop: EventLoop<PomarinEvent>,
    _max_fps: u64,
    _fps_time: Duration,
    initial_size: PhysicalSize<u32>,
}

impl AppUi {
    pub fn new(app_config: AppConfig) -> Self {
        let max_fps = 120;
        let _fps_time = Duration::from_millis(1_000 / max_fps);
        let event_loop = EventLoop::<PomarinEvent>::with_user_event();
        Self {
            app_config,
            event_loop,
            _max_fps: max_fps,
            _fps_time,
            initial_size: PhysicalSize::new(200, 200),
        }
    }

    pub fn get_emitter(&self) -> Arc<Emitter<PomarinEvent>> {
        Arc::new(Emitter::new(&self.event_loop))
    }

    // cannot use ui after run
    pub fn run(self) {
        let window = WindowBuilder::new().build(&self.event_loop).unwrap();
        window.set_title(APP_NAME);
        window.set_decorations(false);
        window.set_maximized(true);
        window.set_visible(true);
        window.set_min_inner_size(Some(self.initial_size));

        let mut wgpu = WgpuState::init(&window, &self.app_config.resources);
        let mut egui = EguiWgpuPassBuilder::new(EguiRoutine::default()).build(
            &wgpu,
            &window,
            &self.event_loop,
        );
        let mut rend = ObjectsPass::new(&wgpu, &window, &self.event_loop);

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
                                wgpu.pre_resize(size);
                                // other
                                rend.resize(&wgpu);
                                wgpu.post_resize(size);

                                log::info!(target: "event", "Window resized to {:?}", size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::UserEvent(event) => {
                    rend.handle_event(event);
                    match event {
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
                    }
                }
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
