mod app;
mod args;
mod client;
mod error;
mod messages;
mod timestrip;
mod utils;

use app::App;
use error::MessageErr;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

fn main() -> Result<(), MessageErr> {
    init_logger(LevelFilter::Debug).unwrap();
    set_default_level(LevelFilter::Trace);

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    app_result
}
