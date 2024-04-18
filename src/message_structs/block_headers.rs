use super::common_traits::csv_format::CSVFormat;
use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::utils::array_tools::{from_le_bytes_i32, from_le_bytes_u32, reverse_array};
use chrono::{prelude::*, LocalResult};
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct BlockHeader {
    pub version: i32,
    pub previous_block_header_hash: [u8; 32],
    pub merkle_root_hash: [u8; 32],
    pub time: u32,
    pub n_bits: u32,
    pub nonce: u32,
}

impl Clone for BlockHeader {
    fn clone(&self) -> BlockHeader {
        Self {
            version: self.version,
            previous_block_header_hash: self.previous_block_header_hash,
            merkle_root_hash: self.merkle_root_hash,
            time: self.time,
            n_bits: self.n_bits,
            nonce: self.nonce,
        }
    }
}

impl BlockHeader {
    pub fn new(
        version: i32,
        previous_block_header_hash: [u8; 32],
        merkle_root_hash: [u8; 32],
        time: u32,
        n_bits: u32,
        nonce: u32,
    ) -> BlockHeader {
        Self {
            version,
            previous_block_header_hash,
            merkle_root_hash,
            time,
            n_bits,
            nonce,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized_header: Vec<u8> = Vec::new();
        serialized_header.extend_from_slice(&self.version.to_le_bytes());
        serialized_header.extend_from_slice(&reverse_array(&self.previous_block_header_hash));
        serialized_header.extend_from_slice(&reverse_array(&self.merkle_root_hash));
        serialized_header.extend_from_slice(&self.time.to_le_bytes());
        serialized_header.extend_from_slice(&self.n_bits.to_le_bytes());
        serialized_header.extend_from_slice(&self.nonce.to_le_bytes());
        serialized_header
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let block_header_serialize = self.serialize();
        let payload = 80;
        let header_block_header = BitcoinMessageHeader::message(
            &block_header_serialize,
            [
                b'b', b'l', b'o', b'c', b'k', b'h', b'e', b'a', b'd', b'e', b'r', b's',
            ],
            payload as u32,
        );
        let header = header_block_header.header(&block_header_serialize);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<BlockHeader, Box<dyn std::error::Error>> {
        if payload.len() >=80{
            let version_deserialize = from_le_bytes_i32(payload);
        let previous_block_header_hash_deserialize =
            match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
                Ok(a) => reverse_array(&a),
                Err(_e) => {
                    return Err("Failed to deserialize".into());
                }
            };
        let merkle_root_hash_deserialize =
            match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
                Ok(a) => reverse_array(&a),
                Err(_e) => {
                    return Err("Failed to deserialize".into());
                }
            };

        let time_deserialize = from_le_bytes_u32(payload);
        let n_bits_deserialize = from_le_bytes_u32(payload);
        let nonce_deserialize = from_le_bytes_u32(payload);

        Ok(BlockHeader::new(
            version_deserialize,
            previous_block_header_hash_deserialize,
            merkle_root_hash_deserialize,
            time_deserialize,
            n_bits_deserialize,
            nonce_deserialize,
        ))
        }
        else{
            Err("header muy corto".into())
        }
    }

    pub fn deserialize_to_vec(
        payload: &mut Vec<u8>,
        count: usize,
    ) -> Result<Vec<BlockHeader>, Box<dyn std::error::Error>> {
        let mut vector: Vec<BlockHeader> = Vec::new();
        for _i in 0..count {
            let block_header = match Self::deserialize(payload) {
                Ok(a) => a,
                Err(_) => {
                    return Err("Failed to deserialize".into());
                }
            };
            vector.push(block_header);
            payload.remove(0);
        }
        Ok(vector)
    }

    pub fn should_download(&self) -> bool {
        let start_date = Utc.with_ymd_and_hms(2023, 4, 10, 9, 0, 0);
        if let LocalResult::Single(s) = start_date {
            let number = s.timestamp() as u32;
            self.time > number
        } else {
            false
        }
    }

    pub fn previous_block_header_hash(&self) -> [u8; 32] {
        reverse_array(&self.previous_block_header_hash)
    }

    pub fn from_string_to_vec(
        string_vector: Vec<Vec<String>>,
    ) -> Result<Vec<BlockHeader>, Box<dyn std::error::Error>> {
        let mut vector: Vec<BlockHeader> = Vec::new();
        for mut i in string_vector {
            let mut block_header_string: Vec<u8> = vec![];
            i.pop();
            for mut j in i {
                match j.as_mut_str().parse() {
                    Ok(v) => block_header_string.extend_from_slice(&[v]),
                    Err(_v) => println!("last line error with block_headers.rs"),
                };
            }
            let block_header = match Self::deserialize(&mut block_header_string) {
                Ok(a) => a,
                Err(_) => {
                    return Err("Failed to deserialize".into());
                }
            };
            println!("header leido");
            vector.push(block_header);
        }
        Ok(vector)
    }
}

impl CSVFormat for BlockHeader {
    fn get_csv_format(&self) -> Vec<String> {
        let version_to_str = self.version.to_string();

        let mut prev_block_header_hash_to_str = String::new();
        for byte in self.previous_block_header_hash.iter() {
            prev_block_header_hash_to_str.push_str(&format!("{:02X}", byte));
        }

        let mut merkle_root_hash_to_str = String::new();
        for byte in self.merkle_root_hash.iter() {
            merkle_root_hash_to_str.push_str(&format!("{:02X}", byte));
        }

        let time_to_str = self.time.to_string();
        let n_bits_to_str = self.n_bits.to_string();
        let nonce_to_str = self.nonce.to_string();

        vec![
            version_to_str,
            prev_block_header_hash_to_str,
            merkle_root_hash_to_str,
            time_to_str,
            n_bits_to_str,
            nonce_to_str,
        ]
    }
}

#[cfg(test)]
mod block_header_tests {

    use super::*;
    use crate::utils::array_tools::cast_str_to_fixed_bytes;

    fn setup() -> BlockHeader {
        let hash_string = "00000000000003a20def7a05a77361b9657ff954b2f2080e135ea6f5970da215";
        let hash_bytes = cast_str_to_fixed_bytes(hash_string).unwrap();

        let merkle_root = "a08f8101f50fd9c9b3e5252aff4c1c1bd668f878fffaf3d0dbddeb029c307e88";
        let merkle_bytes = cast_str_to_fixed_bytes(merkle_root).unwrap();

        let n_bits = "1a05db8b";
        let n_bits_u32 =
            u32::from_str_radix(n_bits, 16).expect("Failed to convert hex string to u32");

        let nonce = "f7d8d840";
        let nonce_u32 =
            u32::from_str_radix(nonce, 16).expect("Failed to convert hex string to u32");

        let header = BlockHeader {
            version: 2,
            previous_block_header_hash: hash_bytes,
            merkle_root_hash: merkle_bytes,
            time: 1348310759,
            n_bits: n_bits_u32,
            nonce: nonce_u32,
        };

        println!("BlockHeader created: {:?}", header);
        header
    }

    #[test]
    fn test_serialize() {
        let header = setup();

        let expected_serialized = [
            2, 0, 0, 0, 21, 162, 13, 151, 245, 166, 94, 19, 14, 8, 242, 178, 84, 249, 127, 101,
            185, 97, 115, 167, 5, 122, 239, 13, 162, 3, 0, 0, 0, 0, 0, 0, 136, 126, 48, 156, 2,
            235, 221, 219, 208, 243, 250, 255, 120, 248, 104, 214, 27, 28, 76, 255, 42, 37, 229,
            179, 201, 217, 15, 245, 1, 129, 143, 160, 231, 150, 93, 80, 139, 219, 5, 26, 64, 216,
            216, 247,
        ];

        assert_eq!(header.serialize(), expected_serialized);
    }

    #[test]
    fn test_deserialize() {
        let header = setup();
        let mut serialized = header.serialize();

        let deserialized = BlockHeader::deserialize(&mut serialized).unwrap();

        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_get_csv_format_version_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(block_header_to_csv[0], "536870912");
    }

    #[test]
    fn test_get_csv_format_prev_hash_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(
            block_header_to_csv[1],
            "B3D7F97A4B4E7F96460A820F4B606C6326EF5F0BB2F216C9010000000000DE23"
        );
    }

    #[test]
    fn test_get_csv_format_merkle_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(
            block_header_to_csv[2],
            "E0E7CCEE1D8A33A8146F5C6BB5AF74EB5C6EE3D708FC97CA7FC4F6C193AD7E5C"
        );
    }

    #[test]
    fn test_get_csv_format_time_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(block_header_to_csv[3], "4283766123");
    }

    #[test]
    fn test_get_csv_format_n_bits_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(block_header_to_csv[4], "555778047");
    }

    #[test]
    fn test_get_csv_format_nonce_ok() {
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

        let block_header_to_csv = block_header.get_csv_format();
        assert_eq!(block_header_to_csv[5], "136530");
    }
}
