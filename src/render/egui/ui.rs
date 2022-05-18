use crate::app::event::{Emitter, EventEmitter, PomarinEvent};
use anyhow::{anyhow, Result};
use egui::Align2;
use std::sync::Arc;
use winit::event_loop::EventLoop;

pub struct EguiRoutine {
    emitter: Option<Arc<Emitter<PomarinEvent>>>,
}

impl Default for EguiRoutine {
    fn default() -> Self {
        Self { emitter: None }
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
        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.close_app().err().map(|e| log::error!("{:?}", e));
                        _frame.quit();
                    }
                });
            });
    }

    fn name(&self) -> &str {
        "test"
    }
}
