use crate::message_structs::block_headers::BlockHeader;
use crate::utils::array_tools::reverse_array;

use super::hashes::header_calculate_doublehash_array_be;

/// Verifies the header given
pub fn verify_header(header: &BlockHeader) -> bool {
    header_check_proof_of_work(header) // && verify_header_links ?
}

/// Check Proof of Work for a given header
pub fn header_check_proof_of_work(header: &BlockHeader) -> bool {
    let mut hash = match header_calculate_doublehash_array_be(header) {
        Some(v) => v,
        None => return false,
    };
    hash = reverse_array(&hash);

    // Calculate the target difficulty as a value to compare against the header hash
    let target_difficulty = match calculate_target_difficulty(header.n_bits) {
        Some(v) => v,
        None => return false,
    };

    // Check if the header hash meets the target difficulty
    // println!("hash: {:x?}", hash);
    // println!("target: {:x?}", target_difficulty);
    hash <= target_difficulty
}

/// Verifies the links of the headers given
fn _verify_header_links(headers: &[BlockHeader]) -> bool {
    // Verify the links between headers
    let mut prev_hash = headers[0].previous_block_header_hash;
    for header in headers.iter().skip(1) {
        if header.previous_block_header_hash != prev_hash {
            return false;
        }
        prev_hash = header.previous_block_header_hash;
    }
    true
}

/// Index Calculation
fn calculate_index(bits: u32) -> u32 {
    bits >> 24
}

/// Coefficient Calculation
fn calculate_coefficient(bits: u32) -> u32 {
    bits & 0xFFFFFF
}

/// Calculates the target difficulty
fn calculate_target_difficulty(bits: u32) -> Option<[u8; 32]> {
    if bits == 0 {
        return None;
    }

    let _ = bits.to_be_bytes();

    let index = calculate_index(bits);
    let coefficient = calculate_coefficient(bits);

    // println!("index: {:?}", index);
    // println!("coefficient: {:?}", coefficient);

    // Calculate the target difficulty
    let target = (coefficient * 256) ^ (index - 3);

    // println!("target: {:?}", target.to_be_bytes());

    let mut target_bytes = [0u8; 32];

    for (i, value) in target_bytes.iter_mut().enumerate().take(3) {
        *value = target.to_be_bytes()[2 - i];
    }

    // println!("target bytes: {:?}", target_bytes);

    Some(target_bytes)
}

#[cfg(test)]

mod headers_validations_tests {

    use crate::utils::array_tools::cast_str_to_fixed_bytes;

    use super::*;

    fn setup() -> BlockHeader {
        let prev_hash_string = "00000000000003a20def7a05a77361b9657ff954b2f2080e135ea6f5970da215";
        let hash_bytes = cast_str_to_fixed_bytes(prev_hash_string).unwrap();

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

    fn setup2() -> BlockHeader {
        // Invalid header. Target is too low
        let invalid_header = BlockHeader {
            version: 1,
            previous_block_header_hash: [9u8; 32],
            merkle_root_hash: [9u8; 32],
            time: 1348310759,
            n_bits: 1000000000,
            nonce: 1348310759,
        };

        println!("Invalid BlockHeader created: {:?}", invalid_header);
        invalid_header
    }

    #[test]
    fn test_calculate_index() {
        let bits = 0x1a05db8b;
        println!("bits: {:?}", bits);
        let result = calculate_index(bits);

        assert_eq!(result, 26)
    }

    #[test]
    fn test_calculate_coefficient() {
        let bits = 0x1a05db8b;
        let result = calculate_coefficient(bits);

        assert_eq!(result, 383883)
    }

    #[test]
    fn test_calculate_target_difficulty1() {
        let bits = 0x181bc330;
        let result = calculate_target_difficulty(bits).unwrap();

        // 1BC330 in little endian
        assert_eq!(
            result,
            [
                48, 195, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0
            ]
        )
    }

    #[test]
    fn test_calculate_target_difficulty2() {
        let bits = 0x1a05db8b;
        let result = calculate_target_difficulty(bits).unwrap();

        // 05DB8B in little endian
        assert_eq!(
            result,
            [
                139, 219, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0
            ]
        )
    }

    #[test]
    fn test_check_proof_of_work_passes() {
        let header = setup();

        assert!(header_check_proof_of_work(&header));
    }

    #[test]
    fn test_check_proof_of_work_fails() {
        let header = setup2();

        assert!(!header_check_proof_of_work(&header));
    }

    #[test]
    fn test_verify_header_links() {
        let header = setup();

        let headers = vec![header];

        assert!(_verify_header_links(&headers));
    }
}
