use crate::protos::{Message::Msg, RequestPayload::ReqPayload};
use protobuf::Message;

fn random_message_id() -> Vec<u8> {
    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(&std::net::Ipv4Addr::LOCALHOST.octets());
    buf.extend_from_slice(&0u16.to_be_bytes());
    buf.extend_from_slice(&rand::random::<u16>().to_be_bytes());
    buf.extend_from_slice(
        &std::time::SystemTime::now()
            .elapsed()
            .unwrap()
            .as_nanos()
            .to_be_bytes(),
    );
    buf
}

fn checksum(message_id: &[u8], payload: &[u8]) -> u64 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(message_id);
    hasher.update(payload);
    hasher.finalize() as u64
}

pub fn wrap_payload(message_id: Vec<u8>, payload: Vec<u8>) -> Msg {
    let checksum = checksum(&message_id, &payload);
    Msg {
        messageID: message_id,
        payload,
        checkSum: checksum,
        special_fields: Default::default(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_checksum() {
        let message_id = vec![1, 2, 3, 4];
        let payload = vec![5, 6, 7, 8];
        assert_eq!(checksum(&message_id, &payload), 1070237893);
    }
}