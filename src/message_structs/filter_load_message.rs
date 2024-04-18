use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct FilterLoadMessage {
    n_filter_bytes: u32,
    filter: Vec<u8>,
    n_hash_funcs: u32,
    n_tweak: u32,
    n_flags: u8,
}

impl FilterLoadMessage {
    pub fn new(
        n_filter_bytes: u32,
        filter: Vec<u8>,
        n_hash_funcs: u32,
        n_tweak: u32,
        n_flags: u8,
    ) -> FilterLoadMessage {
        Self {
            n_filter_bytes,
            filter,
            n_hash_funcs,
            n_tweak,
            n_flags,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut filter_load_message: Vec<u8> = Vec::new();
        filter_load_message.extend_from_slice(&self.n_filter_bytes.to_le_bytes());
        for i in &self.filter {
            filter_load_message.extend_from_slice(&[*i]);
        }
        filter_load_message.extend_from_slice(&self.n_hash_funcs.to_le_bytes());
        filter_load_message.extend_from_slice(&self.n_tweak.to_le_bytes());
        filter_load_message.extend_from_slice(&self.n_flags.to_le_bytes());
        filter_load_message
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_filter_load = self.serialize();
        let payload = 13 + self.filter.len();
        let header_filter_load = BitcoinMessageHeader::message(
            &serialize_filter_load,
            [
                b'f', b'i', b'l', b't', b'e', b'r', b'l', b'o', b'a', b'd', 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_filter_load.header(&serialize_filter_load);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> FilterLoadMessage {
        let n_filter_bytes = Self::from_le_bytes_u32(payload);
        let filter_serilize = payload
            .drain(..(n_filter_bytes as usize))
            .collect::<Vec<u8>>();
        let mut filter: Vec<u8> = Vec::new();
        for i in filter_serilize {
            filter.push(u8::from_le_bytes([i]));
        }
        let n_hash_funcs = Self::from_le_bytes_u32(payload);

        let n_tweak = Self::from_le_bytes_u32(payload);
        let n_flags = u8::from_le_bytes([payload.remove(0)]);

        FilterLoadMessage::new(n_filter_bytes, filter, n_hash_funcs, n_tweak, n_flags)
    }

    fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
        u32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }
}
