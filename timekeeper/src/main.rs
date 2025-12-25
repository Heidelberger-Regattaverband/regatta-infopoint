mod app;
mod args;
mod error;

use ::tui_logger::{init_logger, set_default_level};
use app::App;
use error::TimekeeperErr;

#[tokio::main]
async fn main() -> Result<(), TimekeeperErr> {
    init_logger(tui_logger::LevelFilter::Debug).unwrap();
    set_default_level(tui_logger::LevelFilter::Trace);

    let app = App::new().await?;
    let mut terminal = ratatui::init();
    let app_result = app.start(&mut terminal).await;
    ratatui::restore();

    Ok(app_result?)
}
