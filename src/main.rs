use std::{collections::HashMap, io, net::Ipv4Addr};
use clap::Parser;
use cpen431::server::Server;
use tokio::net::UdpSocket;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(default_value = "0.0.0.0")]
    ip: Ipv4Addr,
    #[arg(default_value = "16401")]
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    Server::new(cli.ip, cli.port).run().await
}
