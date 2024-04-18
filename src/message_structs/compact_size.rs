use super::common_traits::csv_format::CSVFormat;

/// ### Compact Size
/// In the context of Bitcoin and blockchain programming, the `CompactSize` (also known as VarInt or Variable Length Integer) is a format used to represent integers with varying lengths.
/// The CompactSize format allows for the efficient representation of integers that can range from 0 to 2^64 - 1. It achieves this by using a variable number of bytes to represent the value, depending on its magnitude.
/// The format is defined as follows:
/// - If the value is less than 0xFD (253), it is stored as a single byte.
/// - If the value is between 0xFD and 0xFFFF (65,535), it is stored as a 2-byte little-endian value with the leading byte set to 0xFD.
/// - If the value is between 0x10000 (65,536) and 0xFFFFFFFF (4,294,967,295), it is stored as a 4-byte little-endian value with the leading byte set to 0xFE.
/// - If the value is between 0x100000000 (4,294,967,296) and 0xFFFFFFFFFFFFFFFF (18,446,744,073,709,551,615), it is stored as an 8-byte little-endian value with the leading byte set to 0xFF.
/// The CompactSize format is commonly used in various parts of the Bitcoin protocol, such as transaction inputs and outputs, block headers, and more. It allows for efficient encoding and decoding of integers with different lengths while minimizing the space required for storage.

#[derive(PartialEq, Debug)]
pub struct CompactSize {
    pub prefix: u8,
    pub number_vec: Vec<u8>,
    pub number: usize,
}

impl Clone for CompactSize {
    fn clone(&self) -> CompactSize {
        Self {
            prefix: self.prefix,
            number_vec: self.number_vec(),
            number: self.number,
        }
    }
}

impl CompactSize {
    pub fn serialize(&self) -> Vec<u8> {
        let mut compact_size: Vec<u8> = Vec::new();
        if self.prefix != 0 {
            compact_size.extend_from_slice(&[self.prefix]);
        };
        compact_size.extend_from_slice(&self.number_vec);
        compact_size
    }

    pub fn deserialize_u16(payload: &mut Vec<u8>) -> CompactSize {
        let vector: Vec<u8> = vec![payload.remove(0), payload.remove(0)];
        let number = u16::from_le_bytes([vector[0], vector[1]]) as usize;

        CompactSize::from_usize_to_compact_size(number)
    }

    pub fn deserialize_u32(payload: &mut Vec<u8>) -> CompactSize {
        let vector: Vec<u8> = vec![
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ];
        let number = u32::from_le_bytes([vector[0], vector[1], vector[2], vector[3]]);

        CompactSize {
            prefix: 0xFE,
            number_vec: vector,
            number: number as usize,
        }
    }

    pub fn deserialize_u64(payload: &mut Vec<u8>) -> CompactSize {
        let vector: Vec<u8> = vec![
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ];
        let number = u64::from_le_bytes([
            vector[0], vector[1], vector[2], vector[3], vector[4], vector[5], vector[6], vector[7],
        ]);

        CompactSize {
            prefix: 0xFF,
            number_vec: vector,
            number: number as usize,
        }
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> CompactSize {
        let vector: Vec<u8> = vec![payload.remove(0)];
        let compact_size: CompactSize;

        if vector[0] == 0xFD {
            compact_size = Self::deserialize_u16(payload);
        } else if vector[0] == 0xFE {
            compact_size = Self::deserialize_u32(payload);
        } else if vector[0] == 0xFF {
            compact_size = Self::deserialize_u64(payload);
        } else {
            let number = u8::from_le_bytes([vector[0]]);
            compact_size = CompactSize {
                prefix: 0,
                number_vec: vector,
                number: number as usize,
            };
        };
        compact_size
    }

    pub fn size(&self) -> usize {
        let mut answer = self.number_vec.len();
        if self.prefix != 0 {
            answer += 1;
        };
        answer
    }

    pub fn get_number(&self) -> usize {
        self.number
    }

    pub fn get_prefix(&self) -> u8 {
        self.prefix
    }

    pub fn number_vec(&self) -> Vec<u8> {
        let mut vector: Vec<u8> = Vec::new();
        vector.extend_from_slice(&self.number_vec);
        vector
    }

    pub fn from_usize_to_compact_size(number: usize) -> CompactSize {
        let prefix;
        let mut number_vec = (number.to_le_bytes()).to_vec();

        let limit1 = 0xFD;
        let limit2 = 65536;
        let limit3 = 4294967296;
        let limit4 = 18446744073709551615;

        let prefix1 = 0xFD;
        let prefix2 = 0xFE;
        let prefix3 = 0xFF;

        if (limit1 <= number) && (number < limit2) {
            prefix = prefix1;
            number_vec.resize(2, 0);
        } else if (limit2 <= number) && (number < limit3) {
            prefix = prefix2;
            number_vec.resize(4, 0);
        } else if (limit3 <= number) && (number <= limit4) {
            prefix = prefix3;
            number_vec.resize(8, 0);
        } else {
            prefix = 0;
            number_vec.resize(1, 0);
        };
        CompactSize {
            prefix,
            number_vec,
            number,
        }
    }
}

impl CSVFormat for CompactSize {
    /// Only the number_vec is returned as a CSV format
    fn get_csv_format(&self) -> Vec<String> {
        // number_vec
        let mut number_vec_to_csv = Vec::new();
        for n in self.number_vec.iter() {
            number_vec_to_csv.push(n.to_string());
        }

        number_vec_to_csv
    }
}

#[cfg(test)]

mod compact_size_tests {
    use super::CSVFormat;
    use super::CompactSize;

    #[test]
    fn test_compact_size_less_than_0xfd_prefix() {
        let compact_size_vec = vec![100];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.prefix, 0);
    }

    #[test]
    fn test_compact_size_less_than_0xfd_number_vec() {
        let compact_size_vec = vec![100];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.number_vec, compact_size_vec);
    }

    #[test]
    fn test_compact_size_less_than_0xfd_number() {
        let compact_size_vec = vec![100];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.number, 100);
    }

    #[test]
    fn test_compact_size_less_than_0xffff_prefix() {
        let compact_size_vec: Vec<u8> = vec![
            0xFD, 0x00, 0xFE, 0x00, 0xFF, 0x00, // Range: 0xFD00 - 0xFF00
            0xFF, 0x01, 0xFF, 0x02, 0xFF,
            0x03, // Range: 0xFF01 - 0xFF03
                  // Add more values as needed
        ];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.prefix, 0xFD);
    }

    #[test]
    fn test_compact_size_less_than_0xffff_number_vec() {
        let compact_size_vec: Vec<u8> = vec![
            0xFD, 0x00, 0xFE, 0x00, 0xFF, 0x00, // Range: 0xFD00 - 0xFF00
            0xFF, 0x01, 0xFF, 0x02, 0xFF,
            0x03, // Range: 0xFF01 - 0xFF03
                  // Add more values as needed
        ];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.number_vec, [0x00, 0xFE]);
    }

    #[test]
    fn test_compact_size_less_than_0xffff_number() {
        let compact_size_vec: Vec<u8> = vec![
            0xFD, 0x00, 0xFE, 0x00, 0xFF, 0x00, // Range: 0xFD00 - 0xFF00
            0xFF, 0x01, 0xFF, 0x02, 0xFF,
            0x03, // Range: 0xFF01 - 0xFF03
                  // Add more values as needed
        ];
        let deserialized = CompactSize::deserialize(&mut compact_size_vec.clone());

        assert_eq!(deserialized.number, 65024);
    }

    #[test]
    fn test_get_format() {
        let compact_size = CompactSize {
            prefix: 0,
            number_vec: vec![1, 3, 5, 7],
            number: 0,
        };

        let csv_format = compact_size.get_csv_format();

        assert_eq!(csv_format, vec!["1", "3", "5", "7"]);
    }

    #[test]
    fn test_serialize_deserialize() {
        let compact_size = CompactSize::from_usize_to_compact_size(65024);

        let serialized = compact_size.serialize();
        let deserialized = CompactSize::deserialize(&mut serialized.clone());

        assert_eq!(compact_size, deserialized);
    }
}
