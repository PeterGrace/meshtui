extern crate tokio;
#[macro_use] extern crate lazy_static;

pub mod app;
pub mod tui;
pub mod consts;

use tokio::io;
use app::App;


#[tokio::main]
async fn main() -> io::Result<()> {
    App::default().run().await;
    Ok(())
}

