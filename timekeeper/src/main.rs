mod app;
mod args;
mod utils;

use ::aquarius::error::TimekeeperErr;
use ::tui_logger::{init_logger, set_default_level};
use app::App;

#[tokio::main]
async fn main() -> Result<(), TimekeeperErr> {
    init_logger(tui_logger::LevelFilter::Debug).unwrap();
    set_default_level(tui_logger::LevelFilter::Trace);

    let mut terminal = ratatui::init();
    let app_result = App::new().await.start(&mut terminal).await;
    ratatui::restore();

    app_result
}
