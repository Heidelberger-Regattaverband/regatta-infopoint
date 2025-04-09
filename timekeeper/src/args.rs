use clap::{Parser, command};

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser)]
#[command(name = built_info::PKG_NAME)]
#[command(version = built_info::PKG_VERSION)]
#[command(about = built_info::PKG_DESCRIPTION, long_about = None)]
pub(crate) struct Args {
    /// The host to connect to
    #[arg(long, default_value = "127.0.0.1")]
    pub(crate) host: String,

    /// The port to connect to
    #[arg(long, default_value = "2048")]
    pub(crate) port: u16,

    /// The connection timeout in seconds
    #[arg(long, default_value = "1")]
    pub(crate) timeout: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args() {
        let args = Args::parse();
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, 2048);
        assert_eq!(args.timeout, 1);
    }
}
