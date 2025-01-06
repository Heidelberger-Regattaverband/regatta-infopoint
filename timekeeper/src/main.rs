mod app;
mod args;
mod client;
mod error;
mod messages;
mod utils;

use app::App;
use client::{Client, HeatEventReceiver};
use error::MessageErr;
use log::{debug, info};

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();

    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();

    app_result
}
