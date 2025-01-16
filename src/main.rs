use std::net::Ipv4Addr;
use clap::Parser;
use cpen431::{application::{random_message_id, Request, Response}, protocol::Msg};
use protobuf::Message;

fn ping_is_alive(server_ip: Ipv4Addr, student_id: u32, port: u16) -> Option<u32> {
    let retries = 3;
    let default_timeout = std::time::Duration::from_millis(100);

    let message_id = random_message_id(port);
    let request = Msg::from_components(message_id, cpen431::application::Command::IsAlive, None, None, None).write_to_bytes().unwrap();

    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_read_timeout(Some(default_timeout)).unwrap();

    eprintln!("Request: {}", hex::encode(&request));
    for retry in 0..=retries {
        socket.send_to(&request, (server_ip, port)).unwrap();

        let mut response_data = [0; 1024];
        match socket.recv_from(&mut response_data) {
            Ok((size, _)) => {
                let response_data = &response_data[..size];
                let response = Msg::from_bytes(response_data);
                if response.messageID == message_id {
                    let secret_key = response.response().errCode;
                    return Some(secret_key);
                } else {
                    eprintln!("Invalid message ID");
                }
            }
            Err(_) => {
                eprintln!("Timeout")
            }
        }
        socket.set_read_timeout(Some(default_timeout * (1 << retry))).unwrap();
    }    
    None
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    server_ip: Ipv4Addr,
    port: u16,
    student_id: u32,
}

fn main() {
    let cli = Cli::parse();
    let secret_code = ping_is_alive(cli.server_ip, cli.student_id, cli.port).unwrap();
    println!("{}", secret_code);
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test() {
        // Example on Google Doc
        let student_id = 1381632;
        let resp = ping_is_alive(Ipv4Addr::from_str("52.27.39.26").unwrap(), student_id, 43102).unwrap();
        assert_eq!(resp, 0);
    }
}