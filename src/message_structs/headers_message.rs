use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::common_traits::csv_format::CSVFormat;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::inv::Inv;
use crate::message_structs::inv_or_get_data_message::InvOrGetDataMessage;
use crate::node::validation_engine::hashes::header_calculate_doublehash_array_be;
use bitcoin_hashes::{sha256, Hash};
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq, Clone)]
pub struct HeadersMessage {
    count: CompactSize,
    pub headers: Vec<BlockHeader>,
}

// impl Clone for HeadersMessage {
//     fn clone(&self) -> HeadersMessage {
//         Self {
//             count:self.count,
//             headers:self.headers,
//         }
//     }
// }

impl HeadersMessage {
    pub fn new(count: CompactSize, headers: Vec<BlockHeader>) -> HeadersMessage {
        Self { count, headers }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut headers_message: Vec<u8> = Vec::new();
        headers_message.extend_from_slice(&self.count.serialize());
        for i in &self.headers {
            let block_header_serialize = i.serialize();
            for j in block_header_serialize {
                headers_message.extend_from_slice(&[j]);
            }
            headers_message.push(0);
        }
        headers_message
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_headers = self.serialize();
        let payload = self.count.size() + 81 * self.headers.len();
        let header_headers = BitcoinMessageHeader::message(
            &serialize_headers,
            [
                b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_headers.header(&serialize_headers);
        stream.write_all(&header)?;
        match stream.flush() {
            Ok(_) => Ok("HeadersMessage sent"),
            Err(e) => Err(e.into()),
        }
    }

    pub fn deserialize(
        payload: &mut Vec<u8>,
    ) -> Result<HeadersMessage, Box<dyn std::error::Error>> {
        let count = CompactSize::deserialize(payload);

        let headers = match BlockHeader::deserialize_to_vec(payload, count.get_number()) {
            Ok(headers) => headers,
            Err(e) => return Err(e),
        };

        Ok(HeadersMessage::new(count, headers))
    }

    pub fn count(&self) -> usize {
        self.count.get_number()
    }

    pub fn last_hash(&self) -> [u8; 32] {
        let block_header = match self.headers.last() {
            Some(v) => v,
            None => return [0; 32],
        };
        let vector = block_header.serialize();
        let hash = sha256::Hash::hash(&vector);
        let hash2 = sha256::Hash::hash(&hash.to_byte_array());
        hash2.to_byte_array()
    }

    pub fn create_get_data(&self) -> InvOrGetDataMessage {
        let mut inventory: Vec<Inv> = Vec::new();
        let mut count: usize = 0;
        for i in &self.headers {
            if i.should_download() {
                let new_inv = Inv::new(2, i.previous_block_header_hash());
                inventory.push(new_inv);
                count += 1;
            };
        }
        let inv_count = CompactSize::from_usize_to_compact_size(count);
        InvOrGetDataMessage::new(inv_count, inventory)
    }

    pub fn headers_from_str(
        vector: Vec<Vec<String>>,
    ) -> Result<HeadersMessage, Box<dyn std::error::Error>> {
        let count = CompactSize::from_usize_to_compact_size(vector.len());
        let headers = match BlockHeader::from_string_to_vec(vector) {
            Ok(v) => v,
            Err(v) => return Err(v),
        };
        Ok(Self { count, headers })
    }

    pub fn to_serialized(&self)->Option<Vec<Vec<u8>>>{
        if self.headers.is_empty() || self.headers[0] ==BlockHeader::new(0, [0u8; 32], [0u8; 32], 0, 0, 0){
            None
        }
        else{
            let mut vector= vec![];
            for i in self.headers.clone(){
                vector.push(i.serialize());
            }
            Some(vector)
        }
    }

    pub fn get_data_with_type(&self, inv_type: u32) -> Vec<InvOrGetDataMessage> {
        let block_header = match self.headers.last() {
            Some(v) => v,
            None => {
                return Vec::new();
            }
        };
        if !block_header.should_download() {
            return Vec::new();
        }

        let mut vector: Vec<InvOrGetDataMessage> = Vec::new();
        let mut headers_read = 0;
        let mut headers_left = self.count.get_number();
        while headers_left > 0 {
            let mut inventory: Vec<Inv> = Vec::new();
            if self.headers[headers_read].should_download() {
                let hash = header_calculate_doublehash_array_be(&self.headers[headers_read])
                    .unwrap_or([0; 32]);
                let new_inv = Inv::new(inv_type, hash);
                inventory.push(new_inv);
                let inv_count = CompactSize::from_usize_to_compact_size(1);
                vector.push(InvOrGetDataMessage::new(inv_count, inventory));
            }
            headers_read += 1;
            headers_left -= 1;
        }
        vector
    }

    pub fn blocks_missing(self, last_block: BlockHeader) -> HeadersMessage {
        let mut headers_message = self;
        let mut not_found = true;
        let hash = header_calculate_doublehash_array_be(&last_block).unwrap_or([0; 32]);
        if (last_block.previous_block_header_hash == [0; 32]) || (hash == [0; 32]) {
            not_found = false;
        }
        while not_found && !headers_message.headers.is_empty() {
            let i = header_calculate_doublehash_array_be(&headers_message.headers.remove(0))
                .unwrap_or([0; 32]);
            headers_message.count =
                CompactSize::from_usize_to_compact_size(headers_message.count.number - 1);
            if i == hash {
                not_found = false;
            }
        }
        if headers_message.headers.is_empty() {
            HeadersMessage {
                count: CompactSize::from_usize_to_compact_size(1),
                headers: vec![last_block],
            }
        } else {
            headers_message
        }
    }

    pub fn get_headers(&self) -> &Vec<BlockHeader> {
        &self.headers
    }

    pub fn drop_last(&mut self) {
        self.headers.pop();
    }
}

impl CSVFormat for HeadersMessage {
    fn get_csv_format(&self) -> Vec<String> {
        let mut headers_csv = vec![];
        for i in &self.headers {
            headers_csv.push(vec!["{".to_string()]); // isolates headers with {}
            headers_csv.push(i.get_csv_format());
            headers_csv.push(vec!["}".to_string()]); // isolates headers with {}
        }
        headers_csv.into_iter().flatten().collect::<Vec<String>>()
    }
}
