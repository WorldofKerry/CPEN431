use std::net::Ipv4Addr;
use clap::Parser;
use cpen431::{application::{random_message_id, Request, Response}, protocol::Msg};
use protobuf::Message;

fn get_secret_code(server_ip: Ipv4Addr, student_id: u32, port: u16) -> Option<Vec<u8>> {
    let retries = 3;
    let default_timeout = std::time::Duration::from_millis(100);

    let message_id = random_message_id(port);
    let request = Msg::from_student_id(message_id, student_id).write_to_bytes().unwrap();

    let socket = std::net::UdpSocket::bind("0.0.0.0:34254").unwrap();
    socket.set_read_timeout(Some(default_timeout)).unwrap();

    eprintln!("Request: {}", hex::encode(&request));
    for retry in 0..=retries {
        socket.send_to(&request, (server_ip, port)).unwrap();

        let mut response_data = [0; 1024];
        match socket.recv_from(&mut response_data) {
            Ok((size, _)) => {
                let response_data = &response_data[..size];
                let response = Msg::parse_response(response_data);
                if response.messageID == message_id {
                    let secret_key = response.response().secretKey;
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
    let secret_code = get_secret_code(cli.server_ip, cli.student_id, cli.port).unwrap();
    println!("Student ID: {}", cli.student_id);
    println!("Secret Code Length: {}", secret_code.len());
    println!("Secret Code: {}", hex::encode(&secret_code));
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test() {
        // Example on Google Doc
        let student_id = 1381632;
        let actual_secret_code = get_secret_code(Ipv4Addr::from_str("52.27.39.26").unwrap(), student_id, 43102).unwrap();
        let expected_secret_code = hex::decode("D502F4661C49849B2B2FA95A623294BB").unwrap();
        assert_eq!(actual_secret_code, expected_secret_code);
    }
}