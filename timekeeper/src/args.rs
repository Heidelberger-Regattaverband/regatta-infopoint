use clap::Parser;

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
    #[arg(long, default_value = "aquarius")]
    pub(crate) host: String,

    /// The port to connect to
    #[arg(long, default_value = "2048")]
    pub(crate) port: u16,

    /// The connection timeout in seconds
    #[arg(long, default_value = "1")]
    pub(crate) timeout: u16,

    /// The database host
    #[arg(long, default_value = "data")]
    pub(crate) db_host: String,

    /// The database port
    #[arg(long, default_value = "1433")]
    pub(crate) db_port: u16,

    /// The database name
    #[arg(long, default_value = "Regatta_Test")]
    pub(crate) db_name: String,

    /// The database user
    #[arg(long, default_value = "")]
    pub(crate) db_user: String,

    /// The database password
    #[arg(long, default_value = "")]
    pub(crate) db_password: String,
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
