use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::compact_size::CompactSize;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct RejectMessage {
    message_len: CompactSize,
    message: Vec<u8>,
    code: [u8; 12],
    reason_len: CompactSize,
    reason: Vec<u8>,
    extra_data: [u8; 32],
}

impl RejectMessage {
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut reject_message: Vec<u8> = Vec::new();
        reject_message.extend_from_slice(&self.message_len.serialize());
        for i in &self.message {
            reject_message.extend_from_slice(&[*i]);
        }
        reject_message.extend_from_slice(&Self::array_serialize_12(&self.code));
        reject_message.extend_from_slice(&self.reason_len.serialize());
        for i in &self.reason {
            reject_message.extend_from_slice(&i.to_le_bytes());
        }
        reject_message.extend_from_slice(&Self::array_serialize(&self.extra_data));
        reject_message
    }

    /// # Errors
    /// This functions returns an error if the header could not be written to the stream
    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_reject = self.serialize();
        let payload = 44
            + self.message.len()
            + self.reason.len()
            + self.message_len.size()
            + self.reason_len.size();
        let header_reject = BitcoinMessageHeader::message(
            &serialize_reject,
            [
                b'r', b'e', b'j', b'e', b'c', b't', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_reject.header(&serialize_reject);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    /// # Errors
    /// This function will return an error if the payload is not a valid RejectMessage
    pub fn deserialize(payload: &mut Vec<u8>) -> Result<RejectMessage, Box<dyn std::error::Error>> {
        let message_len = CompactSize::deserialize(payload);
        let message_serilize = payload
            .drain(..(message_len.get_number()))
            .collect::<Vec<u8>>();
        let mut message: Vec<u8> = Vec::new();
        for i in message_serilize {
            message.push(u8::from_le_bytes([i]));
        }
        let code = match payload.drain(0..12).collect::<Vec<u8>>().try_into() {
            Ok(a) => Self::array_deserialize_12(a),
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };
        let reason_len = CompactSize::deserialize(payload);
        let reason_serialize = payload
            .drain(..(reason_len.get_number()))
            .collect::<Vec<u8>>();
        let mut reason: Vec<u8> = Vec::new();
        for i in reason_serialize {
            reason.push(u8::from_le_bytes([i]));
        }
        let extra_data = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => Self::array_deserialize(a),
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };

        Ok(RejectMessage {
            message_len,
            message,
            code,
            reason_len,
            reason,
            extra_data,
        })
    }
    fn array_serialize(array: &[u8; 32]) -> [u8; 32] {
        let mut array_new = [0u8; 32];
        for i in 0..32 {
            array_new[i] = array[i].to_le_bytes()[0];
        }
        array_new
    }
    fn array_serialize_12(array: &[u8; 12]) -> [u8; 12] {
        let mut array_new = [0u8; 12];
        for i in 0..12 {
            array_new[i] = array[i].to_le_bytes()[0];
        }
        array_new
    }

    fn array_deserialize(array: [u8; 32]) -> [u8; 32] {
        let mut array_new = [0u8; 32];
        for i in 0..32 {
            array_new[i] = u8::from_le_bytes([array[i]]);
        }
        array_new
    }

    fn array_deserialize_12(array: [u8; 12]) -> [u8; 12] {
        let mut array_new = [0u8; 12];
        for i in 0..12 {
            array_new[i] = u8::from_le_bytes([array[i]]);
        }
        array_new
    }
}
