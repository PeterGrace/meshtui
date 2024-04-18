use clap::Parser;
#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    #[arg(short, long, help="The ip of the host to connect to")]
    pub ip: Option<String>,
    #[arg(short, long, help="The serial port to connect to")]
    pub serial_port: Option<String>,
    #[arg(short, long, help="The tcp port for stream api (defaults to 4403)", default_value_t = 4403)]
    pub tcp_port: u16,
}