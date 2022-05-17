mod app;
mod ui;

use crate::app::{config::load_conf, event::PomarinEvent, render::AppRender};
use std::{thread, time::Duration};

pub const APP_NAME: &'static str = "Pomarin";

// used to specify logs scope
const ENV_FILE: &str = "dev.env";

fn main() {
    println!(
        " --- Starting {} (loading env from {}) --- ",
        APP_NAME, ENV_FILE
    );

    dotenv::from_filename(ENV_FILE).ok();

    env_logger::init();
    log::info!("Initialized environment and logger");

    let ui = AppRender::new(load_conf());
    let emitter = ui.get_emitter_handle();

    thread::spawn(move || loop {
        emitter
            .emit(PomarinEvent::SomeEvent)
            .err()
            .map(|e| log::error!("some event err: {:?}", e));
        thread::sleep(Duration::new(1, 0));
    });

    ui.run();
}
