use std::{sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{event_loop::EventLoop, window::Window};

use crate::{Emitter, PomarinEvent, WgpuRpass, WgpuState};

use super::event::EventEmitter;

pub struct EguiRoutine {
    name: String,
    age: u32,
    emitter: Option<Arc<Emitter<PomarinEvent>>>,
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

// \\ // \\

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

pub struct EguiWgpuPassBuilder<T> {
    gui: T,
}

impl<T> EguiWgpuPassBuilder<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    pub fn new(gui: T) -> Self {
        Self { gui }
    }
}

impl<T> EguiWgpuPassBuilder<T>
where
    T: EventEmitter<PomarinEvent> + epi::App,
{
    pub fn build(
        mut self,
        wgpu: &WgpuState,
        window: &Window,
        event_loop: &EventLoop<PomarinEvent>,
    ) -> EguiWgpuPass<T> {
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

        EguiWgpuPass {
            platform,
            rpass,
            previous_frame_time: None,
            repainter,
            gui: self.gui,
        }
    }
}

// retain egui state
pub struct EguiWgpuPass<T>
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
    fn handle_event(&mut self, event: &winit::event::Event<PomarinEvent>) {
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
