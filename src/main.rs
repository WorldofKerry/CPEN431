use std::{io, net::Ipv4Addr};
use clap::Parser;
use cpen431::server::Server;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    ip: Ipv4Addr,
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_span_events(FmtSpan::NEW)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cli = Cli::parse();
    let mut server = Server::default();
    server.serve(cli.ip, cli.port).await
}
