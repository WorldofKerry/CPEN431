use std::{collections::HashMap, io, net::Ipv4Addr};
use clap::Parser;
use cpen431::{application::{random_message_id, Command, Deserialize, Request, Response, Serialize}, protocol::Msg};
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
    Server::new(cli.ip, cli.port).run().await
}

struct Server {
    ip: Ipv4Addr,
    port: u16,
    kv_data: HashMap<Vec<u8>, Vec<u8>>,
}

impl Server {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Server {
            ip,
            port,
            kv_data: HashMap::new(),
        }
    }

    fn handle_request(&self, request: Request) -> Response {
        match request.command {
            Command::IsAlive => Response { 
                err_code: 0,
                ..Default::default()
            },
            _ => {
                eprintln!("Unsupported command: {:?}", request.command);
                Response {
                    err_code: 1,
                    ..Default::default()
                }
            }
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        let sock = UdpSocket::bind((self.ip, self.port)).await?;
        println!("Server listening on {}:{}", sock.local_addr().unwrap().ip(), sock.local_addr().unwrap().port());
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = sock.recv_from(&mut buf).await?;
            println!("{:?} bytes received from {:?}", len, addr);

            let req_msg = Msg::from_bytes(&buf[..len]);
            let request = req_msg.payload();
            let message_id = req_msg.message_id();
            println!("{:?}", request);

            let response = self.handle_request(request);
            let rsp_msg = response.to_bytes(message_id);
            sock.send_to(&rsp_msg, addr).await?;
        }
    }
}
