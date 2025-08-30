mod app;
mod aquarius;
mod args;
mod error;
mod utils;

use app::App;
use error::TimekeeperErr;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

#[tokio::main]
async fn main() -> Result<(), TimekeeperErr> {
    init_logger(LevelFilter::Debug).unwrap();
    set_default_level(LevelFilter::Trace);

    let mut terminal = ratatui::init();
    let app_result = App::new().await.start(&mut terminal).await;
    ratatui::restore();

    app_result
}
