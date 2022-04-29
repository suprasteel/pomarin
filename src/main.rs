mod render_engine;

use std::{
    thread::{self},
    time::{Duration, Instant},
};

use render_engine::Wgpu;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder,
};

const WINDOW_NAME: &'static str = "Pomarin";

// used to specify logs scope
#[cfg(debug_assertions)]
const ENV_FILE: &str = "dev.env";
#[cfg(not(debug_assertions))]
const ENV_FILE: &str = ".env";

fn main() {
    println!(
        " --- Starting Pomarin Application (loading environment from {}) --- ",
        ENV_FILE
    );
    dotenv::from_filename(ENV_FILE).ok();
    env_logger::init();
    log::info!("Initialized environment and logger");

    let ui = Ui::new(60);
    let emitter = ui.get_event_emitter();

    thread::spawn(move || loop {
        emitter
            .send_event(PomarinEvent::SomeEvent)
            .err()
            .map(|e| log::error!("oops: {:?}", e));
        // log::info!("s");
        thread::sleep(Duration::new(1, 0));
    });

    ui.run();
}

#[derive(Debug, Clone, Copy)]
pub enum PomarinEvent {
    SomeEvent,
}

pub struct Ui {
    event_loop: EventLoop<PomarinEvent>,
    _max_fps: u64,
    fps_time: Duration,
}

impl Ui {
    pub fn new(max_fps: u64) -> Self {
        let fps_time = Duration::from_millis(1_000 / max_fps);
        let event_loop = EventLoop::<PomarinEvent>::with_user_event();
        Self {
            event_loop,
            _max_fps: max_fps,
            fps_time,
        }
    }

    pub fn get_event_emitter(&self) -> EventLoopProxy<PomarinEvent> {
        self.event_loop.create_proxy()
    }

    // duration and measure time
    fn elapsed_time_since_last_frame(last_frame_time: Instant) -> (Duration, Instant) {
        let now = std::time::Instant::now();
        (now - last_frame_time, now)
    }

    // cannot use ui after run
    pub fn run(self) {
        let window = WindowBuilder::new().build(&self.event_loop).unwrap();
        window.set_title(WINDOW_NAME);
        window.set_visible(true);

        // should read config files and load models during init
        let mut engine = pollster::block_on(Wgpu::new(&window));

        log::info!("Starting event loop");
        let mut last_frame_time = std::time::Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } => {
                    if window_id == window.id() {
                        engine.handle_window_event(event);
                        match event {
                            WindowEvent::CloseRequested => {
                                log::info!(target: "event", "Window closed on user request");
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::Resized(phy_size) => {
                                log::info!(target: "event", "Window resized to {:?}", *phy_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::UserEvent(event) => match event {
                    PomarinEvent::SomeEvent => {
                        log::info!(target: "event", "some user event");
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let (dt, now) = Ui::elapsed_time_since_last_frame(last_frame_time);
                    last_frame_time = now;
                    if dt < self.fps_time {
                        thread::sleep(self.fps_time - dt);
                    }
                    engine.render(dt).err().map(|e| log::error!("{}", e));
                }
                _ => {}
            };
        });
    }
}
