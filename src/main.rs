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

use crate::app::Preferences;
use crate::app::{Connection, DeviceConfiguration};
use crate::clap::CliArgs;
use ::clap::Parser;
use app::App;
use lazy_static::lazy_static;

use std::process;

use crate::ipc::IPCMessage;
use ratatui::prelude::*;
use tokio::io;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tui_logger::TuiTracingSubscriberLayer;

lazy_static! {
    static ref PREFERENCES: RwLock<Preferences> = RwLock::new(Preferences::default());
    static ref PAGE_SIZE: RwLock<u16> = RwLock::new(0_u16);
    static ref TO_RADIO_MPSC: RwLock<Option<Sender<IPCMessage>>> = RwLock::new(None);
    static ref FIFTY_FIFTY: Vec<Constraint> =
        vec![Constraint::Percentage(50), Constraint::Percentage(50)];
    static ref DEVICE_CONFIG: RwLock<Option<DeviceConfiguration>> = RwLock::new(None);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    if let Err(_e) = std::env::var("RUST_LOG") {
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
        prefs.initialized = "Yes".to_owned();
        prefs.show_mqtt = cli.show_mqtt;
    }
    assert!(!PREFERENCES.read().await.initialized.is_empty());
    let _ = app.run().await;

    Ok(())
}
