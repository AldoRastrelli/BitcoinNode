use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use bitcoin_hashes::{sha256, sha256d, Hash};

/// Receives a vector. Returns a Hash.
/// For some reason Satoshi decided thath the hash should be return as big endian in Sha256dHash, so here we are.
/// This function will return the hash in big endian.
pub fn vec_calculate_doublehash_be_hash(serialized: &[u8]) -> sha256d::Hash {
    // Compute the double SHA-256 hash with sha256d -> d is for "double"
    sha256d::Hash::hash(serialized)
}

/// Receives a vector. Returns a [u8;32], if possible
/// For some reason Satoshi decided that the hash should be return as big endian in Sha256dHash, so here we are.
/// This function will return the hash in big endian.
pub fn vec_calculate_doublehash_array_be(serialized: Vec<u8>) -> Option<[u8; 32]> {
    // println!("CALCULATE HASH\n-serialized: {:?}", serialized);
    // Compute the double SHA-256 hash with sha256d -> d is for "double"
    let hash = vec_calculate_doublehash_be_hash(&serialized);

    // println!("double hash: {:?}", hash.to_string());

    let hash_bytes: &[u8] = &hash[..];
    let hash_array: Result<[u8; 32], _> = hash_bytes.try_into();

    hash_array.ok()
}

/// Receives a vector, return a [u8;32], if possible
/// Simple Hash is used for Address Conversion in Transactions
pub fn vec_calculate_simple_hash_array_le(serialized: &[u8]) -> Option<[u8; 32]> {
    // println!("CALCULATE SIMPLE HASH\n-serialized: {:?}", serialized);
    // Compute SHA-256 hash
    let hash = sha256::Hash::hash(serialized);

    // println!("simple hash: {:?}", hash);

    let hash_bytes: &[u8] = &hash[..];
    let hash_array: Result<[u8; 32], _> = hash_bytes.try_into();

    hash_array.ok()
}

/// Receives a BlockHeader, return a [u8;32], if possible
/// This function returns the hash in big endian
pub fn header_calculate_doublehash_be(header: &BlockHeader) -> sha256d::Hash {
    let serialized_header = header.serialize();
    vec_calculate_doublehash_be_hash(&serialized_header)
}

/// Receives a BlockHeader, return a [u8;32], if possible
/// This function returns the hash as a vector of bytes in little endian
pub fn header_calculate_doublehash_array_be(header: &BlockHeader) -> Option<[u8; 32]> {
    let serialized = header.serialize();
    vec_calculate_doublehash_array_be(serialized)
}

/// Receives a BlockMessage, return a [u8;32], if possible
/// This function returns the hash in big endian
pub fn block_calculate_doublehash_be(block: &BlockMessage) -> sha256d::Hash {
    let serialized_block = block.serialize_for_hashing();
    vec_calculate_doublehash_be_hash(&serialized_block)
}

/// This function returns the hash as a vector of bytes in little endian
pub fn block_calculate_hash_vector_le(block: &BlockMessage) -> Option<[u8; 32]> {
    let hash = block_calculate_doublehash_be(block);

    // Convert the hash to [u8; 32]
    let hash_bytes: &[u8] = &hash[..];
    let hash_array: Result<[u8; 32], _> = hash_bytes.try_into();

    hash_array.ok()
}

#[cfg(test)]

mod headers_validations_tests {

    use crate::{
        message_structs::{
            compact_size::CompactSize, input::Input, outpoint::Outpoint, output::Output,
            tx_message::TXMessage,
        },
        utils::array_tools::{cast_str_to_fixed_bytes, reverse_array, u8_array_to_hex_string},
    };

    use super::*;

    fn setup_header() -> BlockHeader {
        // Block From
        // https://explorer.btc.com/btc/block/000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf

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

        BlockHeader {
            version: 2,
            previous_block_header_hash: hash_bytes,
            merkle_root_hash: merkle_bytes,
            time: 1348310759,
            n_bits: n_bits_u32,
            nonce: nonce_u32,
        }
    }

    fn setup_block() -> BlockMessage {
        let block_header = setup_header();

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
    fn test_header_calculate_doublehash_be() {
        let header = setup_header();

        let hash = header_calculate_doublehash_be(&header);
        let hash_string = format!("{:x?}", hash);
        // println!("hash_string: {:?}", hash_string);
        let expected = "0x000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf";

        assert_eq!(hash_string, expected)
    }

    #[test]
    fn test_header_calculate_doublehash_array_be() {
        let header = setup_header();

        let hash = header_calculate_doublehash_be(&header);
        let expected = reverse_array(
            &cast_str_to_fixed_bytes(
                "000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf",
            )
            .unwrap(),
        );

        assert_eq!(hash.to_byte_array(), expected)
    }

    #[test]
    fn test_calculate_string_hash_array() {
        let prehashed = "hola";
        let prehashed_bytes = prehashed.as_bytes().to_vec();

        let hash = vec_calculate_simple_hash_array_le(&prehashed_bytes).unwrap();
        // println!("hash_string: {:?}", hash_string);
        let expected = cast_str_to_fixed_bytes(
            "b221d9dbb083a7f33428d7c2a3c3198ae925614d70210e28716ccaa7cd4ddb79",
        )
        .unwrap();

        assert_eq!(hash, expected)
    }

    #[test]
    fn test_calculate_string_double_hash_array() {
        let prehashed = "hola";
        let prehashed_bytes = prehashed.as_bytes().to_vec();
        // println!("prehashed_bytes: {:?}", prehashed_bytes);

        let hash = vec_calculate_doublehash_array_be(prehashed_bytes).unwrap();
        // println!("hash_string: {:?}", hash_string);
        let expected = cast_str_to_fixed_bytes(
            "2f17965a30dbb82d20f6f7d24f2d13c74b715f3445c6a1ea2f64ec40a1b80241",
        )
        .unwrap();

        assert_eq!(hash, expected)
    }

    #[test]
    fn test_calculate_header_hash_vector() {
        // This test was created using this example:
        // https://blockchain-academy.hs-mittweida.de/courses/blockchain-introduction-technical-beginner-to-intermediate/lessons/lesson-13-bitcoin-block-hash-verification/topic/how-to-calculate-and-verify-a-hash-of-a-block/

        let header = setup_header();

        let expected = "bf0e2e13fce62f3a5f15903a177ad6a258a01f164aefed7d4a03000000000000";
        let expected_bytes = cast_str_to_fixed_bytes(expected).unwrap();

        let hash = header_calculate_doublehash_array_be(&header).unwrap();
        // println!("hash: {:?}", hash);
        assert_eq!(hash, expected_bytes)
    }

    #[test]
    fn test_vec_calculate_doublehash_be_hash() {
        let string = "Thisisabouttobehashed";
        let bytes = string.as_bytes().to_vec();

        let expected = "d87c1290f1b861ec00dfce4a879c11e03e5c8779ae71334b15e9266301d39ffb";
        let expected_bytes = cast_str_to_fixed_bytes(expected).unwrap();

        let hash = vec_calculate_doublehash_be_hash(&bytes);
        // println!("hash: {:?}", hash);
        assert_eq!(hash.to_byte_array(), expected_bytes)
    }

    #[test]
    fn test_block_calculate_doublehash_be() {
        let block = setup_block();

        let hash = block_calculate_doublehash_be(&block);
        let hash_string = format!("{:x?}", hash);
        // println!("hash_string: {:?}", hash_string);
        let expected = "0x000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf";
        assert_eq!(hash_string, expected)
    }

    #[test]
    fn test_block_calculate_hash_vector_le() {
        let block = setup_block();

        let hash = block_calculate_hash_vector_le(&block).unwrap();
        let expected = reverse_array(
            &cast_str_to_fixed_bytes(
                "000000000000034a7dedef4a161fa058a2d67a173a90155f3a2fe6fc132e0ebf",
            )
            .unwrap(),
        );
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_vec_calculate_doublehash_array_be_for_concat_hashes() {
        let hash1 = "holaquetal";
        let hash2 = "adios";

        let hash1_bytes = hash1.as_bytes().to_vec();
        let hash2_bytes = hash2.as_bytes().to_vec();
        // println!("hash1_bytes: {:?}", hash1_bytes);
        // println!("hash2_bytes: {:?}", hash2_bytes);

        let concat_hashes = [&hash1_bytes[..], &hash2_bytes[..]].concat();
        // println!("concat_hashes: {:?}", concat_hashes);

        let hash = vec_calculate_doublehash_array_be(concat_hashes).unwrap();
        let hash_string = u8_array_to_hex_string(&hash);
        // println!("hash_string: {:?}", hash_string);

        let expected = "800972e2d2964b6bdf7c0f5f8d1ef1e0890b7d375abd1ce44bacd27a8fcec883";

        assert_eq!(hash_string, expected)
    }
}
