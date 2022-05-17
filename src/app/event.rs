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

// needed by egui
impl epi::backend::RepaintSignal for Emitter<PomarinEvent> {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap()
            .send_event(PomarinEvent::EguiRequestRedraw)
            .ok();
    }
}
