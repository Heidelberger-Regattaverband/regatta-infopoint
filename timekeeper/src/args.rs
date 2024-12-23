use clap::{command, Parser};

#[derive(Parser)]
#[command(name = "TimeKeeper")]
#[command(version = "0.1.0")]
#[command(about = "A Timekeeper for Aquarius", long_about = None)]
pub(crate) struct Args {
    #[arg(long, default_value = "localhost")]
    pub(crate) host: String,
    #[arg(long, default_value = "2048")]
    pub(crate) port: String,
}
