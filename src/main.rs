use std::{default, net::Ipv4Addr, str::FromStr};
use cpen431::{application::{random_message_id, ReqPayload, Request, Response}, protocol::{Msg, Payload}};
use protobuf::Message;

fn get_secret_code(server_ip: Ipv4Addr, student_id: u32, port: u16) -> Option<Vec<u8>> {
    let retries = 3;
    let default_timeout = std::time::Duration::from_millis(100);

    let message_id = random_message_id(port);
    let request = Msg::from_student_id(message_id, student_id).write_to_bytes().unwrap();

    let socket = std::net::UdpSocket::bind("0.0.0.0:34254").unwrap();
    socket.set_read_timeout(Some(default_timeout)).unwrap();

    for retry in 0..=retries {
        println!("{:02X?}", request);
        socket.send_to(&request, (server_ip, port)).unwrap();

        let mut response_data = [0; 1024];
        match socket.recv_from(&mut response_data) {
            Ok((size, _)) => {
                let response_data = &response_data[..size];
                let response = Msg::parse_response(response_data);
                if response.messageID == message_id {
                    return Some(response.response().secretKey);
                } else {
                    dbg!(response.to_string());
                }
            }
            Err(_) => {
                dbg!("timeout");
            }
        }
        socket.set_read_timeout(Some(default_timeout * (1 << retry))).unwrap();
    }    
    None
}

fn main() {
    let secret_code = get_secret_code(Ipv4Addr::from_str("52.27.39.26").unwrap(), 1381632, 43102);
    dbg!(secret_code);
}
