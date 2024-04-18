use crate::node::validation_engine::hashes::vec_calculate_simple_hash_array_le;
use crate::utils::array_tools::cast_array_to_string;
use crate::utils::array_tools::cast_str_to_bytes_vec;
use crate::utils::array_tools::cast_str_to_fixed_bytes;
use crate::utils::array_tools::u8_array_to_hex_string;
use bitcoin_hashes::ripemd160;

use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::error::Error;

pub struct KeysHandler {
    private_key: [u8; 32],
    pub public_key: Vec<u8>,
}

impl Clone for KeysHandler {
    fn clone(&self) -> Self {
        KeysHandler {
            private_key: self.private_key,
            public_key: self.public_key.clone(),
        }
    }
}

impl KeysHandler {
    pub fn new(priv_key: &str) -> Option<KeysHandler> {
        let private_key = match cast_str_to_fixed_bytes(priv_key) {
            Ok(private_key) => private_key,
            Err(_) => return None,
        };

        let public_key = match Self::calculate_pubkey_vec(&private_key) {
            Some(public_key) => public_key,
            None => return None,
        };

        println!("Keys Handler ok");

        Some(KeysHandler {
            private_key,
            public_key,
        })
    }

    pub fn get_private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// retuns the address in string format
    pub fn get_address(&self) -> String {
        if let Ok(address) = Self::encode_pubkey_address(&self.public_key) {
            cast_array_to_string(&address)
        } else {
            "0".to_string()
        }
    }

    /// retuns the address in vec format
    pub fn get_address_vec(&self) -> Vec<u8> {
        if let Ok(address) = Self::encode_pubkey_address(&self.public_key) {
            address
        } else {
            vec![]
        }
    }

    /// returns true if the public key corresponds to the private key given
    pub fn keys_are_valid(public_key: &[u8], private_key: &[u8]) -> bool {
        let derived_public_key = Self::calculate_pubkey_vec(private_key);
        match derived_public_key {
            Some(derived_public_key) => derived_public_key == public_key,
            None => false,
        }
    }

    /// Returns the Public Key in vector format
    pub fn get_pubkey(&self) -> &Vec<u8> {
        &self.public_key
    }

    pub fn get_privkey(&self) -> String {
        u8_array_to_hex_string(&self.private_key)
    }

    /// Returns the pubkey string
    pub fn get_pubkey_string(&self) -> String {
        Self::build_pubkey(&self.private_key).to_string()
    }

    /// Builds the pub key from the private key
    fn build_pubkey(private_key: &[u8]) -> PublicKey {
        let secp = Secp256k1::new();
        // Create a `SecretKey` from the byte array
        let secret_key = SecretKey::from_slice(private_key).expect("32 bytes, within curve order");
        PublicKey::from_secret_key(&secp, &secret_key)
    }

    /// Calculates the public key vector from the private key
    fn calculate_pubkey_vec(private_key: &[u8]) -> Option<Vec<u8>> {
        let public_key = Self::build_pubkey(private_key);

        match cast_str_to_bytes_vec(&public_key.to_string()) {
            Ok(public_key_bytes) => Some(public_key_bytes.to_vec()),
            Err(_) => None,
        }
    }

    /// Calculates the public key hash from the public key
    pub fn calculate_pubkey_hash(pubkey: &Vec<u8>) -> Option<[u8; 20]> {
        let hash = match vec_calculate_simple_hash_array_le(pubkey) {
            Some(hash) => hash,
            None => {
                return None;
            }
        };
        println!("public_key:{:?}", pubkey);
        println!("public_key_hash: {:?}", hash);

        let public_key_hash = ripemd160::Hash::hash(&hash);
        println!("public_key_ripemd: {:?}", public_key_hash);
        Some(public_key_hash.to_byte_array())
    }

    pub fn get_pubkey_address(&self, public_key: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let address = Self::encode_pubkey_address(public_key)?;

        Ok(address)
    }

    fn _encode_pubkey_address_string(pubkey: &Vec<u8>) -> Result<String, Box<dyn Error>> {
        let address = Self::encode_pubkey_address(pubkey)?;

        Ok(cast_array_to_string(&address))
    }

    /// Encodes the public key hash into a string
    fn encode_pubkey_address(pubkey: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        // public key hash calculation
        let public_key_hash = match Self::calculate_pubkey_hash(pubkey) {
            Some(public_key_hash) => public_key_hash,
            None => {
                return Err("Invalid public key hash".into());
            }
        };

        println!("public_key_hash_bytes: {:?}", public_key_hash);

        let encoded_address = Self::encode_hash(&public_key_hash);

        Ok(encoded_address)
    }

    pub fn encode_hash(public_key_hash: &[u8]) -> Vec<u8> {
        // Add an address version byte in front of the hash.
        let network_version = 0x6f; // 0x6f for P2PKH addresses on the Bitcoin testing network (testnet)

        // Create a copy of the version and hash; then hash that twice with SHA256
        let mut data_with_version = Vec::new();
        data_with_version.push(network_version);
        data_with_version.extend(public_key_hash);

        println!("data_with_version: {:?}", data_with_version);

        // double hash with SHA256d
        let checksum = sha256d::Hash::hash(&data_with_version).to_byte_array();

        println!("checksum: {:?}", checksum);

        // Extract the first four bytes from the double-hashed copy. These are used as a checksum to ensure the base hash gets transmitted correctly.
        let mut data_with_checksum = Vec::new();
        data_with_checksum.extend(data_with_version.iter());
        data_with_checksum.extend(&checksum[..4]);

        println!("data_with_checksum: {:?}", data_with_checksum);

        // Append the checksum to the version and hash, and encode it as a base58 string:
        bs58::encode(data_with_checksum).into_vec()
    }

    /// Decodes the address into a public key hash
    pub fn decode_address(encoded_address: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let decoded_address = bs58::decode(encoded_address).into_vec()?;
        let public_key_hash = &decoded_address[1..21];
        Ok(public_key_hash.to_vec())
    }

    /// Decodes the address into a public key hash
    pub fn decode_address_vec(encoded_address: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let public_key_hash = &encoded_address[1..21];
        Ok(public_key_hash.to_vec())
    }
}

#[cfg(test)]

mod transaction_creation_tests {

    use super::*;
    use crate::utils::array_tools::cast_str_to_bytes_vec;
    use crate::utils::array_tools::cast_str_to_fixed_bytes;

    fn setup() -> KeysHandler {
        // Private and public key obtained from: https://privatekeys.pw/key/5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd#public
        // private key hex format
        // public key compressed format
        match KeysHandler::new("5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd") {
            Some(keys_handler) => keys_handler,
            None => panic!("KeysHandler creation failed"),
        }
    }

    fn custom_setup(private_key: &str) -> KeysHandler {
        match KeysHandler::new(private_key) {
            Some(keys_handler) => keys_handler,
            None => panic!("KeysHandler creation failed"),
        }
    }

    #[test]
    fn test_calculate_pubkey() {
        let keys_handler = setup();
        let expected_public_key = cast_str_to_bytes_vec(
            "03da2b61a2d639eac016bc256d5dafcd5e5bdb78b7cf87f0c459e865025254bb5a",
        )
        .unwrap();

        println!("expected_public_key: {:?}", expected_public_key);
        let public_key = keys_handler.get_pubkey();

        assert_eq!(public_key, &expected_public_key);
    }

    #[test]
    fn test_keys_are_valid_true() {
        // Private and public key obtained from: https://privatekeys.pw/key/5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd#public
        // private key hex format
        // public key compressed format
        let private_key = cast_str_to_fixed_bytes(
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd",
        )
        .unwrap();
        let public_key = cast_str_to_bytes_vec(
            "03da2b61a2d639eac016bc256d5dafcd5e5bdb78b7cf87f0c459e865025254bb5a",
        )
        .unwrap();

        assert!(KeysHandler::keys_are_valid(&public_key, &private_key))
    }

    #[test]
    fn test_keys_are_valid_false() {
        // Private and public key obtained from: https://privatekeys.pw/key/5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd#public
        // private key hex format
        // public key compressed format
        let private_key = cast_str_to_fixed_bytes(
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd",
        )
        .unwrap();
        let wrong_public_key = cast_str_to_bytes_vec(
            "000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        assert!(!KeysHandler::keys_are_valid(
            &wrong_public_key,
            &private_key
        ))
    }

    #[test]
    fn test_decode_address() {
        let keys_handler = setup();

        let expected_public_key_hash =
            KeysHandler::calculate_pubkey_hash(&keys_handler.public_key).unwrap();

        let encoded_address = cast_array_to_string(
            &KeysHandler::encode_pubkey_address(&keys_handler.public_key).unwrap(),
        );
        let public_key_hash = KeysHandler::decode_address(&encoded_address).unwrap();

        assert_eq!(public_key_hash, expected_public_key_hash);
    }

    #[test]
    fn test_calculate_pubkey_hash() {
        let keys_handler =
            custom_setup("18e14a7b6a307f426a94f8114701e7c8e774e7f9a47e2c2035db29a206321725");
        let expected_public_key_hash =
            cast_str_to_bytes_vec("f54a5851e9372b87810a8e60cdd2e7cfd80b6e31").unwrap();

        let public_key_hash = KeysHandler::calculate_pubkey_hash(&keys_handler.public_key).unwrap();

        assert_eq!(&public_key_hash.to_vec(), &expected_public_key_hash);
    }

    #[test]
    fn test_encode_address() {
        let keys_handler = setup();
        let expected_encoded_address = "mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc";

        let encoded_address = cast_array_to_string(
            &KeysHandler::encode_pubkey_address(&keys_handler.public_key).unwrap(),
        );

        println!("encoded_address: {:?}", encoded_address);

        assert_eq!(encoded_address, expected_encoded_address);
    }

    #[test]
    fn test_get_tx_address_success() {
        let keys_handler = setup();
        let expected_encoded_address = "mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc";

        let encoded_address = keys_handler.get_address();

        assert_eq!(encoded_address, expected_encoded_address);
    }
}
