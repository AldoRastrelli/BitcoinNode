use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::compact_size::CompactSize;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct GetBlockTxn {
    block_hash: [u8; 32],
    indexes_length: CompactSize,
    indexes: Vec<CompactSize>,
}

impl GetBlockTxn {
    pub fn new(
        block_hash: [u8; 32],
        indexes_length: CompactSize,
        indexes: Vec<CompactSize>,
    ) -> GetBlockTxn {
        Self {
            block_hash,
            indexes_length,
            indexes,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut get_block_txn: Vec<u8> = Vec::new();
        get_block_txn.extend_from_slice(&self.block_hash);
        get_block_txn.extend_from_slice(&self.indexes_length.serialize());
        for i in &self.indexes {
            get_block_txn.extend_from_slice(&i.serialize());
        }
        get_block_txn
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_get_block_txn = self.serialize();
        let mut payload = 32 + self.indexes_length.size();
        for i in &self.indexes {
            payload += i.size();
        }
        let header_get_block_txn = BitcoinMessageHeader::message(
            &serialize_get_block_txn,
            [
                b'g', b'e', b't', b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00,
            ],
            payload as u32,
        );
        let header = header_get_block_txn.header(&serialize_get_block_txn);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<GetBlockTxn, Box<dyn std::error::Error>> {
        let block_hash = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => a,
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };

        let indexes_length = CompactSize::deserialize(payload);

        let mut indexes: Vec<CompactSize> = Vec::new();
        for _i in 0..indexes_length.get_number() {
            let index = CompactSize::deserialize(payload);
            indexes.push(index);
        }

        Ok(GetBlockTxn::new(block_hash, indexes_length, indexes))
    }

    pub fn block_hash(&self)->[u8;32]{
        self.block_hash
    }

    pub fn indexes(&self)->Vec<CompactSize>{
        self.indexes.clone()
    }
}
