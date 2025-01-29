mod app;
mod aquarius;
mod args;
mod error;
mod timestrip;
mod utils;

use app::App;
use error::TimekeeperErr;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

fn main() -> Result<(), TimekeeperErr> {
    init_logger(LevelFilter::Debug).unwrap();
    set_default_level(LevelFilter::Trace);

    let mut terminal = ratatui::init();
    let app_result = App::default().start(&mut terminal);
    ratatui::restore();

    app_result
}
