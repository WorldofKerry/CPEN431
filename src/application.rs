use crate::protos::KeyValueRequest::KVRequest;
use crate::{
    protocol::{MessageID, Msg, Protocol},
    protos::KeyValueResponse::KVResponse,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
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

#[derive(FromPrimitive)]
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

#[derive(Debug)]
pub struct Request {
    pub command: Command,
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
    pub version: Option<i32>,
}


#[derive(Debug, Default)]
pub struct Response {
    pub err_code: u32,
    pub value: Option<Vec<u8>>,
    pub pid: Option<i32>,
    pub version: Option<i32>,
    pub overload_wait_time: Option<i32>,
    pub membership_count: Option<i32>,
}
pub trait Serialize {
    fn to_bytes(self, message_id: MessageID) -> Vec<u8>;
}

impl Serialize for Response {
    fn to_bytes(self, message_id: MessageID) -> Vec<u8> {
        let kvresponse = KVResponse {
            errCode: self.err_code,
            value: self.value,
            pid: self.pid,
            version: self.version,
            overloadWaitTime: self.overload_wait_time,
            membershipCount: self.membership_count,
            special_fields: Default::default(),
        };
        Msg::from_request(message_id, kvresponse.write_to_bytes().unwrap()).write_to_bytes().unwrap()
    }
}

pub trait Deserialize {
    fn from_bytes(response: &[u8]) -> Self;
    fn payload(&self) -> Request;
    fn message_id(&self) -> MessageID;
}

impl Deserialize for Msg {
    fn from_bytes(response: &[u8]) -> Self {
        Msg::parse_from_bytes(response).unwrap()
    }

    fn payload(&self) -> Request {
        let kvrequest = KVRequest::parse_from_bytes(&self.payload).unwrap();
        Request {
            command: Command::from_u32(kvrequest.command).unwrap(),
            key: kvrequest.key,
            value: kvrequest.value,
            version: kvrequest.version,
        }
    }

    fn message_id(&self) -> MessageID {
        self.messageID.as_slice().try_into().unwrap()
    }
}
