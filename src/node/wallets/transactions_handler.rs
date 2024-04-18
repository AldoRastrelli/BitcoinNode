use super::keys_handler::KeysHandler;
use crate::message_structs::{
    compact_size::CompactSize, input::Input, outpoint::Outpoint, output::Output,
    tx_message::TXMessage,
};
use crate::utils::array_tools::u8_vec_to_hex_string;
use crate::utils::script_tools::from_adderss_to_vec;
use secp256k1::{ecdsa::Signature, Message, Secp256k1, SecretKey};
use std::error::Error as Err;
type TransactionResult = Result<(TXMessage, Vec<(Outpoint, Output)>), Box<dyn Err>>;

/// P2PKH Transaction Handler
pub struct P2PKH {}

impl Default for P2PKH {
    fn default() -> P2PKH {
        P2PKH::new()
    }
}

impl Clone for P2PKH {
    fn clone(&self) -> Self {
        P2PKH::new()
    }
}

impl P2PKH {
    /// Creates a new P2PKH instance
    fn new() -> P2PKH {
        P2PKH {}
    }

    /// Receives the keys_handler, a recipient_address, amount and fee; and returns the corresponding P2PKH Transaction
    pub fn create_transaction(
        keys_handler: &KeysHandler,
        utxos: &[(Outpoint, Output)],
        recipient_address: &[u8],
        amount: i64,
        fee: i64,
    ) -> TransactionResult {
        println!("wallet P2PKH CREATE TRANSACTION");
        // Setup
        let sender_pubkey = keys_handler.get_pubkey();
        let sender_address = keys_handler.get_address();

        println!(
            "Our Address:{:?}",
            bs58::encode(sender_address.clone()).into_string()
        );

        let (utxos, change) = Self::get_utxos_needed(utxos, amount + fee, sender_pubkey);

        // Version
        let version = 1;

        // Inputs and Inputs count
        let (inputs, scripts_used) = Self::create_transaction_inputs(&utxos, sender_pubkey);
        let inputs_count = CompactSize::from_usize_to_compact_size(inputs.len());

        let outputs = Self::create_transaction_outputs(
            amount,
            fee,
            change as i64,
            &from_adderss_to_vec(&sender_address).unwrap(),
            recipient_address,
        );
        let outputs_count = CompactSize::from_usize_to_compact_size(outputs.len());

        println!("wallet P2PKH CREATE TRANSACTION 2");
        // Locktime
        let locktime = 0; //SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        println!("wallet P2PKH CREATE TRANSACTION 3");

        // Create transaction
        let mut transaction = TXMessage::new(
            version,
            inputs_count,
            inputs,
            outputs_count,
            outputs,
            locktime,
        );

        // Sign transaction
        Self::sign_transaction(&mut transaction, keys_handler, scripts_used);

        // Change inputs' script
        // for input in transaction.input_list.iter_mut() {
        //     input.update_script(signature_script.remove(0));

        // }

        println!("wallet P2PKH CREATE TRANSACTION 4: {:?}", transaction);
        println!(
            "tx in hex: {:?}",
            u8_vec_to_hex_string(&transaction.serialize())
        );
        // Signed_transaction
        Ok((transaction, utxos))
    }

    /// Returns the minimum number of UTXOs needed to cover the amount + fee and the value of the change, if any
    fn get_utxos_needed(
        utxos: &[(Outpoint, Output)],
        value: i64,
        _sender_pubkey: &[u8],
    ) -> (Vec<(Outpoint, Output)>, u32) {
        // Sort utxos by ascending order of value
        let mut sorted_utxos = utxos.to_owned();
        sorted_utxos.sort_by(|(_, output1), (_, output2)| output1.value.cmp(&output2.value));

        // Get the minimum number of UTXOs needed to cover the amount + fee
        let mut needed_utxos = Vec::new();
        let mut total_value = 0;

        for (outpoint, output) in sorted_utxos {
            total_value += output.value;
            needed_utxos.push((
                Outpoint::new(outpoint.get_hash(), outpoint.get_index()),
                output.clone(),
            ));
            if total_value >= value {
                break;
            }
        }

        (needed_utxos, (total_value - value) as u32)
    }

    /// Returns the transactino inputs for the utxos passed as arguments.
    /// The scripts and script_lengths are empty, waiting for the Transaction to be signed
    fn create_transaction_inputs(
        utxos: &[(Outpoint, Output)],
        _sender_pubkey: &[u8],
    ) -> (Vec<Input>, Vec<Vec<u8>>) {
        let mut inputs = Vec::new();
        let mut _i = 0;
        let mut scripts = vec![];

        for (_i, (outpoint, output)) in utxos.iter().enumerate() {
            let previous_tx = outpoint;
            scripts.push(output.get_script());
            // Create script and script_lenght
            let script = Vec::new();
            let script_length = CompactSize::from_usize_to_compact_size(0);

            let sequence = 0xffffffff;

            let input = Input::new(*previous_tx, script_length, script, sequence);
            inputs.push(input);
        }
        (inputs, scripts)
    }

    fn create_transaction_outputs(
        value: i64,
        _fee: i64,
        change: i64,
        sender_address: &[u8],
        recipient_address: &[u8],
    ) -> Vec<Output> {
        // If you want to spend less money than the amount available in a UTXO (Unspent Transaction Output), you need to create two outputs in your transaction:
        // Output for the recipient: This output contains the desired amount of money you want to send to the recipient. It specifies the recipient's address and the amount.
        // Change output: This output sends the remaining amount back to yourself. It ensures that the total input amount matches the total output amount in the transaction.

        let mut outputs = Vec::new();

        // Create an Output for recipient_address with value
        let script = Self::output_script_creation_for(recipient_address);
        let script_length = CompactSize::from_usize_to_compact_size(script.len());

        let output = Output::new(value, script_length, script);
        outputs.push(output);

        // Create an Output for fee
        // let script = Self::output_script_fee_creation();
        // let script_length = CompactSize::from_usize_to_compact_size(script.len());

        // let output = Output::new(fee, script_length, script);
        // outputs.push(output);

        // Create an Output for sender_pubkey with change
        let script = Self::output_script_creation_for(sender_address);
        let script_length = CompactSize::from_usize_to_compact_size(script.len());

        let output = Output::new(change, script_length, script);
        outputs.push(output);

        outputs
    }

    /// Output script for recipient or self (returning change). Must receive a pubkey address
    fn output_script_creation_for(address: &[u8]) -> Vec<u8> {
        let mut hash = address.to_owned();

        hash.remove(0); // remove version
        let checksum_pos = hash.len() - 4;
        hash = hash[0..checksum_pos].to_vec(); // remove checksum

        let mut script = vec![];
        // OP_DUP
        script.push(118);

        // OP_HASH160
        script.push(169);
        script.push(hash.len() as u8);
        script.extend_from_slice(&hash);

        // OP_EQUALVERIFY
        script.push(136);

        // OP_CHECKSIG
        script.push(172);
        script
    }

    /// Output script for fee
    fn _output_script_fee_creation() -> Vec<u8> {
        // OP_RETURN
        let script = vec![106];
        script
    }

    /// Signs the transaction and returns the signature serialized
    fn sign_transaction(
        transaction: &mut TXMessage,
        keys_handler: &KeysHandler,
        mut scripts: Vec<Vec<u8>>,
    ) {
        //let mut signatures = vec![];
        println!(
            "before adding script {:?}",
            u8_vec_to_hex_string(&transaction.serialize())
        );
        let mut x = 0;
        while !scripts.is_empty() {
            let mut tx_clone = transaction.clone();
            println!("script:{:?}", scripts[0]);
            //scripts.remove(0);
            tx_clone.update_empty_script(scripts.remove(0));
            //let tx = transaction.clone();
            // let mut tx_seriaized = tx_clone.serialize();
            // //tx_seriaized.extend_from_slice(&[1,0,0,0]);
            // //tx_seriaized.reverse();
            // let mut tx_hash = match calculate_hash_for_serialized(tx_seriaized) {
            //     Some(a) => a,
            //     None => return,
            // };
            //println!("before adding sig {:?}",u8_vec_to_hex_string(&tx_clone.serialize()));
            //tx_hash.reverse();
            // encriptar usando la private key
            let signature = Self::generate_ecdsa_signature(
                keys_handler.get_private_key(),
                &tx_clone.sig_hash(x),
            );
            let pub_key = keys_handler.public_key.clone();
            // signature + public key -> input script
            let mut input_script = Vec::new();
            input_script.extend_from_slice(&[signature.len() as u8 + 1]);
            input_script.extend_from_slice(&signature);
            input_script.push(1_u8.to_be());
            input_script.extend_from_slice(&[keys_handler.public_key.len() as u8]);
            input_script.extend_from_slice(&pub_key);

            transaction.update_empty_script(input_script);
            x += 1;
        }
    }

    /// Input signature script creation
    fn generate_ecdsa_signature(private_key: &[u8], tx_hash: &[u8]) -> Vec<u8> {
        // Create a Secp256k1 context
        let secp = Secp256k1::new();

        // Parse the private key
        let secret_key = SecretKey::from_slice(private_key).expect("Invalid private key");

        // Create a message from the transaction hash
        let message = Message::from_slice(tx_hash).expect("Invalid transaction hash");

        // Sign the message with the private key, producing a recoverable signature
        let recoverable_signature: Signature = secp.sign_ecdsa(&message, &secret_key);

        // Convert the recoverable signature to a regular signature
        let signature = recoverable_signature.serialize_der();

        // Serialize the signature and recovery ID into a byte array
        signature.to_vec()
    }
}

#[cfg(test)]

mod transactions_handler_test {

    use super::*;

    #[test]
    fn test_create_transaction() {
        let keys_handler =
            KeysHandler::new("5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd")
                .unwrap();
        let _sender_address = keys_handler.clone().public_key;
        let recipient_address = &[
            110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117,
            117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87,
        ];

        let value = 2;
        let fee = 1;

        let utxos = vec![
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    5,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![3u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    3,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![2u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
        ];

        let (transaction, _) =
            P2PKH::create_transaction(&keys_handler, &utxos, recipient_address, value, fee)
                .unwrap();

        println!("transaction: {:?}", transaction);

        assert_eq!(transaction.get_input().len(), 2);
        assert_eq!(transaction.get_output().len(), 2);
    }

    #[test]
    fn test_get_utxos_needed_size_ok() {
        let utxos = vec![
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    5,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![3u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    3,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![2u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
        ];

        let (utxos_needed, _) = P2PKH::get_utxos_needed(
            &utxos,
            3,
            &[
                110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57,
                117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87,
            ],
        );
        assert_eq!(utxos_needed.len(), 2);
    }

    #[test]
    fn test_get_utxos_needed_change_ok() {
        let utxos = vec![
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    5,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![3u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    3,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![2u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
        ];

        let (_, change) = P2PKH::get_utxos_needed(
            &utxos,
            3,
            &[
                110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57,
                117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87,
            ],
        );
        assert_eq!(change, 1);
    }

    #[test]
    fn test_create_transaction_inputs() {
        let utxos = vec![
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    5,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![3u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    3,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![2u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
            (
                Outpoint::new([0u8; 32], 0),
                Output::new(
                    2,
                    CompactSize::from_usize_to_compact_size(34),
                    vec![1u8; 34],
                ),
            ),
        ];

        let inputs = P2PKH::create_transaction_inputs(
            &utxos,
            &[
                110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57,
                117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87,
            ],
        );
        println!("{:?}", inputs);
    }

    #[test]
    fn test_create_transaction_outputs() {
        let outputs = P2PKH::create_transaction_outputs(
            2,
            1,
            1,
            &[
                110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57,
                117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87,
            ],
            &[1u8; 34],
        );

        let expected_outputs = [
            Output {
                value: 2,
                script_length: CompactSize {
                    prefix: 0,
                    number_vec: vec![34],
                    number: 34,
                },
                script: vec![
                    118, 169, 29, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 136, 172,
                ],
            },
            Output {
                value: 1,
                script_length: CompactSize {
                    prefix: 0,
                    number_vec: vec![34],
                    number: 34,
                },
                script: vec![
                    118, 169, 29, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110,
                    84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 136, 172,
                ],
            },
        ];

        assert_eq!(outputs, expected_outputs);
    }

    #[test]
    fn test_output_script_creation_for() {
        let keys_handler =
            KeysHandler::new("18e14a7b6a307f426a94f8114701e7c8e774e7f9a47e2c2035db29a206321725")
                .unwrap();
        let address = keys_handler.get_address_vec();
        let address_clone = address;
        // address = [110, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87]

        // delete version byte: [51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 51, 105, 88, 87]
        // delete checksum bytes: [51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75]

        // expected = []
        // expected + OP_DUP = [118]
        // expected + OP_HASH160 = [118, 169]
        // expected + address.len() =  [118, 169, 29]
        // expected + address = [118, 169, 29, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75]
        // expected + OP_EQUALVERIFY = [118, 169, 29, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 136]
        // expected + OP_CHECKSIG = [118, 169, 29, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84, 57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 136, 172]

        let expected_script = vec![
            118, 169, 29, 51, 115, 118, 117, 100, 104, 109, 55, 98, 116, 54, 106, 51, 110, 84, 84,
            57, 117, 117, 49, 65, 53, 55, 67, 115, 57, 112, 75, 75, 136, 172,
        ];

        let script = P2PKH::output_script_creation_for(&address_clone);
        assert_eq!(script, expected_script);
    }

    #[test]
    fn test_output_script_fee_creation() {
        let script = P2PKH::_output_script_fee_creation();
        assert_eq!(script, vec![106]);
    }
}
