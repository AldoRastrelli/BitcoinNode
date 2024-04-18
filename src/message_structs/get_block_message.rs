use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;

use crate::message_structs::compact_size::CompactSize;
use crate::utils::configs::config::get_protocol_version;

#[derive(Debug, PartialEq)]
pub struct GetBlockMessage {
    pub version: u32,
    pub hash_count: CompactSize,
    pub block_hash: Vec<[u8; 32]>,
    pub hash_stop: [u8; 32],
}

impl GetBlockMessage {
    pub fn build_default() -> Result<GetBlockMessage, Box<dyn Error>> {
        let protocol_version = match get_protocol_version() {
            Ok(protocol_version) => protocol_version,
            Err(e) => return Err(e),
        };

        Ok(GetBlockMessage {
            version: protocol_version,
            hash_count: CompactSize {
                prefix: 0,
                number_vec: vec![1],
                number: 1,
            },
            block_hash: vec![[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x9d, 0x66, 0x8c, 0x08, 0x5a, 0xe1, 0x65, 0x83,
                0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6,
                0x0a, 0x8c, 0xe2, 0x6f,
            ]],
            hash_stop: [0u8; 32],
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut get_block_message: Vec<u8> = Vec::new();
        get_block_message.extend_from_slice(&self.version.to_le_bytes());
        get_block_message.extend_from_slice(&self.hash_count.serialize());
        for i in &self.block_hash {
            get_block_message.extend_from_slice(i);
        }
        get_block_message.extend_from_slice(&self.hash_stop);
        get_block_message
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_get_block = self.serialize();
        let payload = self.hash_count.size() + 36 + 32 * self.hash_count.get_number();
        let header_get_block = BitcoinMessageHeader::message(
            &serialize_get_block,
            [
                b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_get_block.header(&serialize_get_block);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(
        payload: &mut Vec<u8>,
    ) -> Result<GetBlockMessage, Box<dyn std::error::Error>> {
        let version = Self::from_le_bytes_u32(payload);

        let hash_count = CompactSize::deserialize(payload);
        let mut block_hash: Vec<[u8; 32]> = Vec::new();
        for _i in 0..hash_count.get_number() {
            let hash = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
                Ok(a) => a,
                Err(_) => {
                    return Err("Failed to deserialize".into());
                }
            };
            block_hash.push(hash);
        }
        let hash_stop = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => a,
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };

        Ok(GetBlockMessage {
            version,
            hash_count,
            block_hash,
            hash_stop,
        })
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
