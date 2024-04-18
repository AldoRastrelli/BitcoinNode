use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::io::Write;
use std::net::TcpStream;

use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::compact_size::CompactSize;

#[derive(Debug, PartialEq, Clone)]
pub struct MerkleBlock {
    pub block_header: BlockHeader,
    pub transaction_count: u32,
    hash_count: CompactSize,
    pub hashes: Vec<[u8; 32]>,
    flag_byte_count: CompactSize,
    pub flags: Vec<u8>,
}

impl MerkleBlock {
    pub fn new(
        block_header: BlockHeader,
        transaction_count: u32,
        hash_count: CompactSize,
        hashes: Vec<[u8; 32]>,
        flag_byte_count: CompactSize,
        flags: Vec<u8>,
    ) -> MerkleBlock {
        Self {
            block_header,
            transaction_count,
            hash_count,
            hashes,
            flag_byte_count,
            flags,
        }
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        let mut merkel_block: Vec<u8> = Vec::new();
        let block_header_serialize = self.block_header.serialize();
        merkel_block.extend_from_slice(&block_header_serialize);
        merkel_block.extend_from_slice(&self.transaction_count.to_le_bytes());
        merkel_block.extend_from_slice(&self.hash_count.serialize());
        for i in &self.hashes {
            merkel_block.extend_from_slice(i);
        }
        merkel_block.extend_from_slice(&self.flag_byte_count.serialize());
        for i in &self.flags {
            merkel_block.extend_from_slice(&i.to_le_bytes());
        }
        merkel_block
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_merkel_block = self.serialize();
        let payload = 84
            + 32 * self.hashes.len()
            + self.hash_count.size()
            + self.flag_byte_count.size()
            + self.flags.len();
        let header_merkel_block = BitcoinMessageHeader::message(
            &serialize_merkel_block,
            [
                b'm', b'e', b'r', b'k', b'l', b'e', b'b', b'l', b'o', b'c', b'k', 0x00,
            ],
            payload as u32,
        );
        let header = header_merkel_block.header(&serialize_merkel_block);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<MerkleBlock, Box<dyn std::error::Error>> {
        let block_header = match BlockHeader::deserialize(payload) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        let transaction_count = Self::from_le_bytes_u32(payload);
        let hash_count = CompactSize::deserialize(payload);
        let mut hashes: Vec<[u8; 32]> = Vec::new();
        for _i in 0..hash_count.get_number() {
            let hash = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
                Ok(a) => a,
                Err(_e) => panic!("Failed to convert vector to array"),
            };
            hashes.push(hash);
        }
        let flag_byte_count = CompactSize::deserialize(payload);
        let mut flags: Vec<u8> = Vec::new();
        for _i in 0..flag_byte_count.get_number() {
            flags.push(payload.remove(0));
        }
        Ok(MerkleBlock::new(
            block_header,
            transaction_count,
            hash_count,
            hashes,
            flag_byte_count,
            flags,
        ))
    }

    pub fn from_string_to_vec(
        vector: Vec<String>,
    ) -> Result<MerkleBlock, Box<dyn std::error::Error>> {
        let mut block_header_string: Vec<u8> = vec![];
        for mut i in vector {
            match i.as_mut_str().parse() {
                Ok(v) => block_header_string.extend_from_slice(&[v]),
                Err(_v) => (),
            };
        }
        //println!("lectura:{:?}",block_header_string);
        let block = match Self::deserialize(&mut block_header_string) {
            Ok(a) => a,
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };
        Ok(block)
    }

    pub fn get_ids(&self) -> Vec<[u8; 32]> {
        let mut vector: Vec<[u8; 32]> = vec![];
        for i in &self.hashes {
            vector.push(*i);
        }
        vector
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
