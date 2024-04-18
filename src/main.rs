extern crate tokio;
#[macro_use] extern crate tracing;

pub mod app;
pub mod tui;
pub mod consts;
mod tabs;
mod theme;
mod meshtastic_interaction;
mod ipc;
mod packet_handler;
mod util;
mod menutabs;

use tokio::io;
use app::App;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;

use tui_logger::TuiTracingSubscriberLayer;

#[tokio::main]
async fn main() -> io::Result<()> {
    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(TuiTracingSubscriberLayer);
    tracing::subscriber::set_global_default(collector).expect("Could not initialize logging.");
    App::default().run().await;
    Ok(())
}

