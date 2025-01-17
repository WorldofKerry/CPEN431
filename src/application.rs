use crate::protos::KeyValueRequest::KVRequest;
use crate::{
    protocol::{MessageID, Msg, Protocol},
    protos::KeyValueResponse::KVResponse,
};
use protobuf::Message;
use std::net::IpAddr;

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

#[derive(Debug)]
pub enum Command {
    Put = 0x01,
    Get = 0x02,
    Remove = 0x03,
    Shutdown = 0x04,
    Wipeout = 0x05,
    IsAlive = 0x06,
    GetPID = 0x07,
    GetMembershipCount = 0x08,
    GetMembershipList = 0x22,
}

impl From<u32> for Command {
    fn from(value: u32) -> Self {
        match value {
            0x01 => Command::Put,
            0x02 => Command::Get,
            0x03 => Command::Remove,
            0x04 => Command::Shutdown,
            0x05 => Command::Wipeout,
            0x06 => Command::IsAlive,
            0x07 => Command::GetPID,
            0x08 => Command::GetMembershipCount,
            0x22 => Command::GetMembershipList,
            _ => panic!("Invalid command"),
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub command: Command,
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
    pub version: Option<i32>,
}

pub trait Serialize {
    fn from_components(
        message_id: MessageID,
        command: Command,
        key: Option<Vec<u8>>,
        value: Option<Vec<u8>>,
        version: Option<i32>,
    ) -> Self;
}

impl Serialize for Msg {
    fn from_components(
        message_id: MessageID,
        command: Command,
        key: Option<Vec<u8>>,
        value: Option<Vec<u8>>,
        version: Option<i32>,
    ) -> Self {
        let payload = KVRequest {
            command: command as u32,
            key,
            value,
            version,
            special_fields: Default::default(),
        };
        Msg::from_request(message_id, payload.write_to_bytes().unwrap())
    }
}

pub trait Deserialize {
    fn from_bytes(response: &[u8]) -> Self;
    fn payload(&self) -> Request;
}

impl Deserialize for Msg {
    fn from_bytes(response: &[u8]) -> Self {
        Msg::parse_from_bytes(response).unwrap()
    }

    fn payload(&self) -> Request {
        let kvrequest = KVRequest::parse_from_bytes(&self.payload).unwrap();
        Request {
            command: Command::from(kvrequest.command),
            key: kvrequest.key,
            value: kvrequest.value,
            version: kvrequest.version,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_msg() {
        let message_id = random_message_id(34254);
        let _request = Msg::from_components(message_id, Command::IsAlive, None, None, None);
    }
}
