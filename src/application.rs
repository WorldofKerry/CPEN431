use crate::{protocol::{MessageID, Msg, Payload}, protos::ResponsePayload::ResPayload};
pub use crate::protos::RequestPayload::ReqPayload;
use protobuf::Message;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn random_message_id(port: u16) -> MessageID {
    let mut message_id: MessageID = [0; 16];
    let client_ip_first_four: [u8; 4] = match local_ip_address::local_ip().unwrap() {
        IpAddr::V4(ipv4) => ipv4.octets(),
        IpAddr::V6(ipv6) => ipv6.octets()[..4].try_into().unwrap(),
    };
    let port_bytes: [u8; 2] = port.to_be_bytes();
    let random_bytes: [u8; 2] = rand::random();
    let time_bytes: [u8; 8] = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64)
        .to_be_bytes();
    message_id[..4].copy_from_slice(&client_ip_first_four);
    message_id[4..6].copy_from_slice(&port_bytes);
    message_id[6..8].copy_from_slice(&random_bytes);
    message_id[8..16].copy_from_slice(&time_bytes);
    message_id
}

pub trait Request {
    fn from_student_id(message_id: MessageID, student_id: u32) -> Self;
}

impl Request for Msg {
    fn from_student_id(message_id: MessageID, student_id: u32) -> Self {
        let payload = ReqPayload {
            studentID: student_id,
            special_fields: Default::default(),
        };
        Msg::from_request(message_id, payload)
    }
}
pub trait Response {
    fn parse_response(response: &[u8]) -> Self;
    fn response(&self) -> ResPayload;
}

impl Response for Msg {
    fn parse_response(response: &[u8]) -> Self {
        Msg::parse_from_bytes(response).unwrap()
    }
    
    fn response(&self) -> ResPayload {
        ResPayload::parse_from_bytes(&self.payload).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_msg() {
        let message_id = random_message_id(34254);
        let request = Msg::from_student_id(message_id, 123456);
    }
}