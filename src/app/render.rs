use std::{sync::Arc, time::Duration};

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    render::{
        egui::{pass::EguiWgpuPass, ui::EguiRoutine},
        scene::pass::ObjectsPass,
        state::WgpuState,
    },
    APP_NAME,
};

use super::{
    config::AppConfig,
    event::{Emitter, PomarinEvent},
};

/// App render manager.
///
/// This struct takes a config from which it defines the window and its content.
///
/// This struct's `run()` method starts a winit event loop with a scene and an egui UI.
///
/// To notify the rendering with app event, get an emitter with `get_emitter_handle()` and call
/// `emit(app_event)` on it.
///
pub struct AppRender {
    app_config: AppConfig,
    event_loop: EventLoop<PomarinEvent>,
    initial_size: PhysicalSize<u32>,
    _max_fps: u64,
    _fps_time: Duration,
}

impl AppRender {
    pub fn new(app_config: AppConfig) -> Self {
        let max_fps = 120;
        let _fps_time = Duration::from_millis(1_000 / max_fps);
        let event_loop = EventLoop::<PomarinEvent>::with_user_event();
        Self {
            app_config,
            event_loop,
            initial_size: PhysicalSize::new(200, 200),
            _max_fps: max_fps,
            _fps_time,
        }
    }

    /// Get the event emitter (Arc wrapped) that will enable sending app event to the event loop
    pub fn get_emitter_handle(&self) -> Arc<Emitter<PomarinEvent>> {
        Arc::new(Emitter::new(&self.event_loop))
    }

    /// Run the winit event loop.
    /// Once this loop is started, it will be closed either by a `PomarinEvent::CloseApp` or a `Event::CloseRequested` event.
    pub fn run(self) {
        let window = WindowBuilder::new().build(&self.event_loop).unwrap();
        window.set_title(APP_NAME);
        window.set_decorations(false);
        window.set_maximized(true);
        window.set_visible(true);
        window.set_min_inner_size(Some(self.initial_size));

        // wgpu state
        let mut wgpu = WgpuState::init(&window, &self.app_config.resources);
        // render egui ui
        let mut egui = EguiWgpuPass::new(&wgpu, &window, &self.event_loop, EguiRoutine::default());
        // render 3d scene
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
                        match event {
                            WindowEvent::CloseRequested => {
                                log::info!(target: "event", "Window CloseRequest");
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(size) => {
                                let size = *size;
                                wgpu.pre_resize(size);
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

                    // the last one wins, we want the ui above the 3d scene
                    let encoder = rend.render(&wgpu, &window, &output_view, encoder);
                    let encoder = egui.render(&wgpu, &window, &output_view, encoder);
                    wgpu.queue.submit(std::iter::once(encoder.finish()));

                    // Redraw
                    output_frame.present();
                }
                _ => {}
            };
        });
    }
}
