use super::common_traits::csv_format::CSVFormat;
use crate::utils::array_tools::reverse_array;
use std::error::Error;
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Outpoint {
    hash: [u8; 32],
    index: u32,
}

impl Outpoint {
    pub fn new(hash: [u8; 32], index: u32) -> Outpoint {
        Self { hash, index }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut outpoint: Vec<u8> = Vec::new();
        outpoint.extend_from_slice(&reverse_array(&self.hash));
        outpoint.extend_from_slice(&self.index.to_le_bytes());
        outpoint
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<Outpoint, Box<dyn Error>> {
        let hash = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => reverse_array(&a),
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };
        let index = Self::from_le_bytes_u32(payload);

        Ok(Outpoint { hash, index })
    }

    pub fn get_hash(&self) -> [u8; 32] {
        reverse_array(&self.hash)
    }
    pub fn get_index(&self) -> u32 {
        self.index
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

impl CSVFormat for Outpoint {
    fn get_csv_format(&self) -> Vec<String> {
        //     hash:[u8;32],
        // index:i32,
        let mut hex_string = String::with_capacity(64);
        for byte in self.hash {
            let hex_byte = format!("{:X}", byte);
            hex_string.push_str(&hex_byte);
        }

        vec![hex_string, self.index.to_string()]
    }
}

#[cfg(test)]

mod outpoint_tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let outpoint = Outpoint::new([0; 32], 0);

        let outpoint_serialized = outpoint.serialize();
        let mut outpoint_serialized_expected = Vec::new();
        outpoint_serialized_expected.extend_from_slice(&[0; 32]);
        outpoint_serialized_expected.extend_from_slice(&0i32.to_le_bytes());
        assert_eq!(outpoint_serialized, outpoint_serialized_expected);
    }

    #[test]
    fn test_deserialize() {
        let mut outpoint_serialized = Vec::new();
        outpoint_serialized.extend_from_slice(&[1; 32]);
        outpoint_serialized.extend_from_slice(&0i32.to_le_bytes());

        let deserialized = Outpoint::deserialize(&mut outpoint_serialized).unwrap();
        let deserialized_expected = Outpoint::new([1; 32], 0);

        assert_eq!(deserialized, deserialized_expected);
    }

    #[test]
    fn test_get_csv_format() {
        let outpoint = Outpoint::new([1; 32], 0);
        let outpoint_csv = outpoint.get_csv_format();
        let outpoint_csv_expected = [
            "11111111111111111111111111111111".to_string(),
            "0".to_string(),
        ];
        assert_eq!(outpoint_csv, outpoint_csv_expected);
    }
}
