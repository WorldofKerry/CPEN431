pub use crate::protos::Message::Msg;
use protobuf::Message;
use crate::application::Result;

pub type MessageID = [u8; 16];

fn checksum(message_id: &[u8], payload: &[u8]) -> u64 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(message_id);
    hasher.update(payload);
    u64::from(hasher.finalize())
}

pub trait Protocol {
    fn from_request(message_id: MessageID, payload: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;
}

impl Protocol for Msg {
    fn from_request(message_id: MessageID, payload: Vec<u8>) -> Self {
        let checksum = checksum(&message_id, &payload);
        Msg {
            messageID: message_id.to_vec(),
            payload,
            checkSum: checksum,
            special_fields: Default::default(),
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        self.write_to_bytes().unwrap()
    }    
    fn from_bytes(bytes: &[u8]) -> Result<Msg> {
        let msg = Msg::parse_from_bytes(bytes)?;
        match msg.checkSum == checksum(&msg.messageID, &msg.payload) {
            true => Ok(msg),
            false => Err(crate::application::ApplicationError::InvalidChecksum),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_checksum() {
        // Example on Google Doc
        let message_id = vec![1, 2, 3, 4];
        let payload = vec![5, 6, 7, 8];
        assert_eq!(checksum(&message_id, &payload), 1070237893);
    }
}
