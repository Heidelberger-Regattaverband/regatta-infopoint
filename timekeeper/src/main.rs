mod app;
mod args;
mod client;
mod error;
mod messages;
mod utils;

use app::App;
use error::MessageErr;

fn main() -> Result<(), MessageErr> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();

    app_result
}
