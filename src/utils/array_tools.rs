use std::error::Error;

/// reverses an array of 32 bytes
pub fn reverse_array(array: &[u8; 32]) -> [u8; 32] {
    let mut reversed_array = [0u8; 32];
    for (i, &element) in array.iter().rev().enumerate() {
        reversed_array[i] = element;
    }
    reversed_array
}

/// casts a little endian vec into i32
pub fn from_le_bytes_i32(payload: &mut Vec<u8>) -> i32 {
    // Each version of a block starts with its version in the most significant 4 bits, the rest of the 0011 is 4,0010 is 3
    i32::from_le_bytes([
        payload.remove(0),
        payload.remove(0),
        payload.remove(0),
        payload.remove(0),
    ])
}

/// casts a little endian vec into u32
pub fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
    u32::from_le_bytes([
        payload.remove(0),
        payload.remove(0),
        payload.remove(0),
        payload.remove(0),
    ])
}

/// Casts an array to a string
pub fn cast_array_to_string(array: &[u8]) -> String {
    let mut string = String::new();
    for &element in array {
        string.push(element as char);
    }
    string
}

/// casts a string to bytes
pub fn cast_str_to_bytes(s: &str) -> Result<&[u8], Box<dyn Error>> {
    let bytes = s.as_bytes();
    Ok(bytes)
}

/// casts a string to bytes in vec type in hex pairs
pub fn cast_str_to_bytes_vec(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();

    let mut iter = s.chars().peekable();
    while let Some(first) = iter.next() {
        let second = iter.next();

        let pair = match second {
            Some(second) => format!("{}{}", first, second),
            None => return Err("Invalid length".into()),
        };

        match u8::from_str_radix(&pair, 16) {
            Ok(byte) => {
                bytes.push(byte);
            }
            Err(_) => return Err("Invalid byte".into()),
        }
    }

    Ok(bytes)
}

/// casts a string to fixed bytes (32). Used for hashes
pub fn cast_str_to_fixed_bytes(s: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if s.len() != 64 {
        return Err("Invalid length".into());
    }

    let mut bytes = [0u8; 32];
    let mut pairs = Vec::new();

    for i in (0..64).step_by(2) {
        let pair = match s.get(i..i + 2) {
            Some(p) => p,
            None => break,
        };

        match u8::from_str_radix(pair, 16) {
            Ok(byte) => {
                pairs.push(byte);
            }
            Err(_) => return Err("Invalid byte".into()),
        };
    }

    bytes.copy_from_slice(&pairs[..]);
    Ok(bytes)
}

/// casts an array to hex string
pub fn u8_array_to_hex_string(array: &[u8; 32]) -> String {
    let mut hex_string = String::with_capacity(64);
    for &byte in array.iter() {
        let hex_byte = format!("{:02x}", byte);
        hex_string.push_str(&hex_byte);
    }
    hex_string
}

pub fn u8_vec_to_hex_string(array: &[u8]) -> String {
    let mut hex_string = String::with_capacity(array.len() * 2);
    for &byte in array.iter() {
        let hex_byte = format!("{:02x}", byte);
        hex_string.push_str(&hex_byte);
    }
    hex_string
}

pub fn cast_vec_to_fixed_array(vec: Vec<u8>) -> [u8; 32] {
    let mut array = [0u8; 32];
    array.copy_from_slice(&vec[..]);
    array
}

/// Casts a hex value vector to a binary array
pub fn cast_vec_to_binary_vec(dec_values: Vec<u8>) -> Vec<u8> {
    let mut binary_bits = Vec::new();

    for value in dec_values {
        let hex_value = format!("{:0x}", value);
        let digits = hex_value.to_string().len() * 4;

        // Convert each hexadecimal value to binary
        let binary = format!("{:0width$b}", value, width = digits);

        // Reverse the binary string and convert each character to a u8
        let bits = binary.chars().rev().map(|c| c.to_digit(2).unwrap() as u8);

        // Extend the vector with the binary bits
        binary_bits.extend(bits);
    }

    binary_bits
}

/// casts u32 to le bytes
pub fn u32_to_le_bytes(value: u32) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(&value.to_le_bytes());
    bytes
}

/// casts u128 to le bytes
pub fn u128_to_le_bytes(value: u128) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..4].copy_from_slice(&value.to_le_bytes());
    bytes
}

#[cfg(test)]

mod array_tools_tests {

    use super::*;

    #[test]
    fn test_cast_string_to_bytes() {
        let hash_str = "bf0e2e13fce62f3a5f15903a177ad6a258a01f164aefed7d4a03000000000000";

        let bytes = cast_str_to_fixed_bytes(hash_str).unwrap();

        let expected_bytes = [
            191, 14, 46, 19, 252, 230, 47, 58, 95, 21, 144, 58, 23, 122, 214, 162, 88, 160, 31, 22,
            74, 239, 237, 125, 74, 3, 0, 0, 0, 0, 0, 0,
        ];

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_reverse_array() {
        let array = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let reversed_array = reverse_array(&array);

        let expected_array = [
            32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11,
            10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
        ];

        assert_eq!(reversed_array, expected_array);
    }

    #[test]
    fn test_u32_to_le_bytes() {
        let value = 123456789;
        let bytes = u32_to_le_bytes(value);

        let expected_bytes = [
            21, 205, 91, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];

        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_cast_string_to_bytes_vec() {
        let string = "f54a5851e9372b87810a8e60cdd2e7cfd80b6e31";
        let expected_bytes = [
            245, 74, 88, 81, 233, 55, 43, 135, 129, 10, 142, 96, 205, 210, 231, 207, 216, 11, 110,
            49,
        ]
        .to_vec();

        let bytes = cast_str_to_bytes_vec(string).unwrap();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_cast_vec_to_binary_vec_1() {
        let hex_values = vec![7];
        let binary = cast_vec_to_binary_vec(hex_values);

        let expected_binary = vec![1, 1, 1, 0];

        assert_eq!(binary, expected_binary);
    }

    #[test]
    fn test_cast_vec_to_binary_vec_2() {
        let hex_values = vec![29]; // 0x1d
        let binary = cast_vec_to_binary_vec(hex_values);

        let expected_binary = vec![1, 0, 1, 1, 1, 0, 0, 0];

        assert_eq!(binary, expected_binary);
    }

    #[test]
    fn test_cast_vec_to_binary_vec_3() {
        let hex_values = vec![7, 0x1d];
        let binary = cast_vec_to_binary_vec(hex_values);

        let expected_binary = vec![1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0];

        assert_eq!(binary, expected_binary);
    }

    #[test]
    fn test_cast_vec_to_binary_vec_4() {
        let hex_values = vec![255, 255, 29]; // 0xff
        let binary = cast_vec_to_binary_vec(hex_values);

        let expected_binary = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0,
        ];

        assert_eq!(binary, expected_binary);
    }
}
