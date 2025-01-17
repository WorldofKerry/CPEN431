use std::{io, net::Ipv4Addr};
use clap::Parser;
use cpen431::{application::{random_message_id, Serialize, Deserialize}, protocol::Msg};
use protobuf::Message;
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
    run_server(cli.ip, cli.port).await
}

async fn run_server(ip: Ipv4Addr, port: u16) -> io::Result<()> {    
    let sock = UdpSocket::bind((ip, port)).await?;
    println!("Trying using debugger more instead of println");
    println!("Server listening on {}:{}", sock.local_addr().unwrap().ip(), sock.local_addr().unwrap().port());
    let mut buf = [0; 1024];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);

        let len = sock.send_to(&buf[..len], addr).await?;
        println!("{:?} bytes sent", len);
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::*;

    #[cfg(skip)]
    #[tokio::test]
    async fn test() {
        let _ = run_server(Ipv4Addr::from_str("0.0.0.0").unwrap(), 16401).await;
    }
}
