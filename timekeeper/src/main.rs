mod app;
mod args;
mod client;
mod error;
mod messages;
mod utils;

use app::App;
use error::MessageErr;

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();

    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();

    app_result
} // the stream is closed here
