extern crate tokio;
#[macro_use]
extern crate tracing;

pub mod app;
mod clap;
pub mod consts;
mod ipc;
mod meshtastic_interaction;
mod packet_handler;
mod tabs;
mod theme;
pub mod tui;
mod util;

use std::env::VarError;
use crate::app::Connection;
use crate::app::Preferences;
use crate::clap::CliArgs;
use ::clap::Parser;
use app::App;
use lazy_static::lazy_static;
use std::process;
use tokio::io;
use tokio::sync::RwLock;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tui_logger::TuiTracingSubscriberLayer;

lazy_static! {
    static ref PREFERENCES: RwLock<Preferences> = RwLock::new(Preferences::default());
}
#[tokio::main]
async fn main() -> io::Result<()> {
    if let Err(e) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }

    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(TuiTracingSubscriberLayer);
    tracing::subscriber::set_global_default(collector).expect("Could not initialize logging.");
    let cli = CliArgs::parse();
    let mut app = App::default();
    if cli.ip.is_some() {
        app.connection = Connection::TCP(cli.ip.unwrap(), cli.tcp_port);
    } else if cli.serial_port.is_some() {
        app.connection = Connection::Serial(cli.serial_port.unwrap());
    } else {
        println!("You must specify an ip via -i, or a serial port v ia -s.");
        process::exit(1);
    }

    {
        let mut prefs = PREFERENCES.write().await;
        // setting this to a nonzero length String to help indicate we're a bona-fide
        // preferences struct and not a ::default() generated one.
        prefs.initialized = "Yes".to_string();
        prefs.show_mqtt = cli.show_mqtt;
    }
    let _ = app.run().await;

    Ok(())
}
