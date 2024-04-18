use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::compact_size::CompactSize;

use std::io::Write;
use std::net::TcpStream;

use super::common_traits::csv_format::CSVFormat;
use crate::message_structs::tx_message::TXMessage;

#[derive(Debug, PartialEq)]
pub struct BlockMessage {
    pub block_header: BlockHeader,
    pub tx_count: CompactSize,
    pub transaction_history: Vec<TXMessage>,
}

impl Clone for BlockMessage {
    fn clone(&self) -> BlockMessage {
        Self {
            block_header: self.block_header.clone(),
            tx_count: self.tx_count.clone(),
            transaction_history: self.transaction_history.clone(),
        }
    }
}

impl BlockMessage {
    pub fn new(
        block_header: BlockHeader,
        tx_count: CompactSize,
        transaction_history: Vec<TXMessage>,
    ) -> BlockMessage {
        Self {
            block_header,
            tx_count,
            transaction_history,
        }
    }

    pub fn get_block_header(&self) -> BlockHeader {
        self.block_header.clone()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut block_message: Vec<u8> = Vec::new();
        let block_header_serialize = self.block_header.serialize();
        block_message.extend_from_slice(&block_header_serialize);
        block_message.extend_from_slice(&self.tx_count.serialize());
        for i in &self.transaction_history {
            let transaction_serialize = i.serialize();
            for j in transaction_serialize {
                block_message.extend_from_slice(&[j]);
            }
        }
        block_message
    }

    pub fn serialize_for_hashing(&self) -> Vec<u8> {
        self.block_header.serialize()
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_block = self.serialize();
        let payload = serialize_block.len() as u32;
        // for i in &self.transaction_history {
        //     payload += i.size();
        // }
        let header_block = BitcoinMessageHeader::message(
            &serialize_block,
            [
                b'b', b'l', b'o', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload,
        );
        let header = header_block.header(&serialize_block);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<BlockMessage, Box<dyn std::error::Error>> {
        let block_header_deserialize = match BlockHeader::deserialize(payload) {
            Ok(block_header_deserialize) => block_header_deserialize,
            Err(e) => return Err(e),
        };

        let tx_count = CompactSize::deserialize(payload);
        let transaction_history_deserialize =
            match TXMessage::deserialize_to_vec(payload, tx_count.get_number()) {
                Ok(transaction_history_deserialize) => transaction_history_deserialize,
                Err(e) => return Err(e),
            };

        Ok(BlockMessage::new(
            block_header_deserialize,
            tx_count,
            transaction_history_deserialize,
        ))
    }
    pub fn get_tx(&self) -> Vec<TXMessage> {
        let mut vector: Vec<TXMessage> = vec![];
        for i in &self.transaction_history {
            let Ok(tx) =TXMessage::deserialize(&mut i.serialize()) else {panic!("problemas con la tx")};
            vector.push(tx);
        }
        vector
    }

    pub fn get_ids(&self) -> Vec<[u8; 32]> {
        let mut vector: Vec<[u8; 32]> = vec![];
        for i in &self.transaction_history {
            let id = i.get_id();
            vector.push(id);
        }
        vector
    }

    pub fn from_string_to_vec(
        vector: Vec<String>,
    ) -> Result<BlockMessage, Box<dyn std::error::Error>> {
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

    pub fn blocks_from_str(vector: Vec<String>) -> BlockMessage {
        let block: BlockMessage = match BlockMessage::from_string_to_vec(vector) {
            Ok(v) => v,
            Err(_v) => BlockMessage {
                block_header: BlockHeader {
                    version: 0,
                    previous_block_header_hash: [0; 32],
                    merkle_root_hash: [0; 32],
                    time: 0,
                    n_bits: 0,
                    nonce: 0,
                },
                tx_count: CompactSize {
                    prefix: 0,
                    number_vec: [0].to_vec(),
                    number: 0,
                },
                transaction_history: vec![TXMessage::new(
                    0,
                    CompactSize {
                        prefix: 0,
                        number_vec: [0].to_vec(),
                        number: 0,
                    },
                    vec![],
                    CompactSize {
                        prefix: 0,
                        number_vec: [0].to_vec(),
                        number: 0,
                    },
                    vec![],
                    0,
                )],
            },
        };
        block
    }
}

impl CSVFormat for BlockMessage {
    fn get_csv_format(&self) -> Vec<String> {
        // block header
        let block_header_to_string_vec = self.block_header.get_csv_format();
        let block_header_to_string = format!("[{}]", block_header_to_string_vec.join(","));

        // transaction history
        let mut transaction_history_to_string_vec = Vec::new();

        for transaction in &self.transaction_history {
            // For each output, we join the items with ',' and then we add '[' and ']' to the string to isolate single outputs from each other
            let transaction_vec = transaction.get_csv_format();

            let mut transaction_to_string = transaction_vec.join(",");
            transaction_to_string = format!("[{}]", transaction_to_string);
            // Add the output to the final vector
            transaction_history_to_string_vec.push(transaction_to_string);
        }

        // Join every outut with ',' and then add '[' and ']' to represent a list of outputs
        let transaction_history_to_string =
            format!("[{}]", transaction_history_to_string_vec.join(","));

        vec![block_header_to_string, transaction_history_to_string]
    }
}

#[cfg(test)]

mod block_message_tests {
    use super::*;
    use crate::message_structs::{
        input::Input, outpoint::Outpoint, output::Output, tx_message::TXMessage,
    };

    fn setup_function() -> BlockMessage {
        let block_header = BlockHeader {
            version: 536870912,
            previous_block_header_hash: [
                179, 215, 249, 122, 75, 78, 127, 150, 70, 10, 130, 15, 75, 96, 108, 99, 38, 239,
                95, 11, 178, 242, 22, 201, 1, 0, 0, 0, 0, 0, 222, 35,
            ],
            merkle_root_hash: [
                224, 231, 204, 238, 29, 138, 51, 168, 20, 111, 92, 107, 181, 175, 116, 235, 92,
                110, 227, 215, 8, 252, 151, 202, 127, 196, 246, 193, 147, 173, 126, 92,
            ],
            time: 4283766123,
            n_bits: 555778047,
            nonce: 136530,
        };

        let input = Input::new(
            Outpoint::new([0u8; 32], 0),
            CompactSize::from_usize_to_compact_size(4),
            vec![1, 3, 5, 7],
            9,
        );

        let output1 = Output::new(100, CompactSize::from_usize_to_compact_size(2), vec![1, 2]);
        let output2 = Output::new(100, CompactSize::from_usize_to_compact_size(2), vec![1, 2]);

        let tx_message = TXMessage::new(
            1,
            CompactSize::from_usize_to_compact_size(1),
            vec![input],
            CompactSize::from_usize_to_compact_size(2),
            vec![output1, output2],
            1,
        );

        BlockMessage::new(
            block_header,
            CompactSize::from_usize_to_compact_size(1),
            vec![tx_message],
        )
    }

    #[test]
    fn test_serialize_deserialize() {
        let block_message = setup_function();
        let mut serialized = block_message.serialize();

        let deserialized = BlockMessage::deserialize(&mut serialized).unwrap();

        println!("{:?}", block_message);
        println!("{:?}", deserialized);

        assert_eq!(block_message, deserialized);
    }

    #[test]
    fn test_serialize_for_hashing() {
        let block_message = setup_function();
        let serialized_for_hashing = block_message.serialize_for_hashing();

        let header_serialized = block_message.block_header.serialize();

        assert_eq!(serialized_for_hashing, header_serialized);
    }
}
