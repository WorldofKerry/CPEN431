pub use crate::protos::Message::Msg;
use protobuf::Message;
use crate::protos::RequestPayload::ReqPayload;

pub type MessageID = [u8; 16];

fn checksum(message_id: &[u8], payload: &[u8]) -> u64 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(message_id);
    hasher.update(payload);
    hasher.finalize() as u64
}

pub trait Payload {
    fn from_request(message_id: MessageID, payload: ReqPayload) -> Self;
}

impl Payload for Msg {
    fn from_request(message_id: MessageID, payload: ReqPayload) -> Self {
        let payload = payload.write_to_bytes().unwrap();
        let checksum = checksum(&message_id, &payload);
        Msg {
            messageID: message_id.to_vec(),
            payload,
            checkSum: checksum,
            special_fields: Default::default(),
        }
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
