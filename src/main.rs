mod actor;
mod packet;

use color_eyre::Report;
use tracing_subscriber;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Report> {
    setup()?;

    return Ok(());
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    color_eyre::install().unwrap();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    return Ok(());
}
