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
mod clap;
use ::clap::Parser;
use tokio::io;
use app::App;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use std::process;

use tui_logger::TuiTracingSubscriberLayer;
use crate::app::Connection;
use crate::clap::CliArgs;
#[tokio::main]
async fn main() -> io::Result<()> {
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
    let _ = app.run().await;

    Ok(())
}

