use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::io::Write;
use std::net::TcpStream;

use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::tx_message::TXMessage;

#[derive(Debug, PartialEq)]
pub struct BlockTxn {
    pub block_hash: [u8; 32],
    pub transactions_length: CompactSize,
    pub transactions: Vec<TXMessage>,
}

impl BlockTxn {
    pub fn new(
        block_hash: [u8; 32],
        transactions_length: CompactSize,
        transactions: Vec<TXMessage>,
    ) -> BlockTxn {
        Self {
            block_hash,
            transactions_length,
            transactions,
        }
    }

    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut block_txn: Vec<u8> = Vec::new();
        block_txn.extend_from_slice(&self.block_hash);
        block_txn.extend_from_slice(&self.transactions_length.serialize());
        for i in &self.transactions {
            let transacrion_serialize = i.serialize();
            for j in transacrion_serialize {
                block_txn.extend_from_slice(&[j]);
            }
        }
        block_txn
    }

    // # Errors
    // This fails when the stream is closed before the message is sent
    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_block_txn = self.serialize();
        let mut payload = 32 + self.transactions_length.size();
        for i in &self.transactions {
            payload += i.size() as usize;
        }
        let header_block_txn = BitcoinMessageHeader::message(
            &serialize_block_txn,
            [
                b'b', b'l', b'o', b'c', b'k', b't', b'x', b'n', 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_block_txn.header(&serialize_block_txn);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    // # Errors
    // This fails when the stream is closed before the message is sent
    pub fn deserialize(payload: &mut Vec<u8>) -> Result<BlockTxn, Box<dyn std::error::Error>> {
        let block_hash: [u8; 32] = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => a,
            Err(_e) => return Err("Error al deserializar block_txn".into()),
        };

        let transactions_length = CompactSize::deserialize(payload);

        let transactions =
            match TXMessage::deserialize_to_vec(payload, transactions_length.get_number()) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };

        Ok(BlockTxn::new(block_hash, transactions_length, transactions))
    }
}
