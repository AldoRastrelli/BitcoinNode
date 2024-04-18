use super::common_traits::csv_format::CSVFormat;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::outpoint::Outpoint;
use std::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct Input {
    previous_tx: Outpoint,
    signature_script_length: CompactSize,
    signature_script: Vec<u8>,
    sequence_number: u32,
}

impl Input {
    pub fn new(
        previous_tx: Outpoint,
        signature_script_length: CompactSize,
        signature_script: Vec<u8>,
        sequence_number: u32,
    ) -> Input {
        Self {
            previous_tx,
            signature_script_length,
            signature_script,
            sequence_number,
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&self.previous_tx.serialize());
        input.extend_from_slice(&self.signature_script_length.serialize());
        for i in &self.signature_script {
            input.extend_from_slice(&[*i]);
        }
        input.extend_from_slice(&self.sequence_number.to_le_bytes());

        input
    }

    pub fn size(&self) -> u32 {
        let mut size = 40 + self.signature_script_length.size();
        for _i in &self.signature_script {
            size += 1;
        }
        size as u32
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<Input, Box<dyn Error>> {
        let previous_tx = match Outpoint::deserialize(payload) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };
        let signature_script_length = CompactSize::deserialize(payload);
        let signature_script_serialize = payload
            .drain(..(signature_script_length.get_number()))
            .collect::<Vec<u8>>();
        let mut signature_script: Vec<u8> = Vec::new();
        for i in signature_script_serialize {
            signature_script.push(u8::from_le_bytes([i]));
        }

        let sequence_number = Self::from_le_bytes_u32(payload);

        Ok(Input::new(
            previous_tx,
            signature_script_length,
            signature_script,
            sequence_number,
        ))
    }

    pub fn deserialize_to_vec(
        payload: &mut Vec<u8>,
        count: u32,
    ) -> Result<Vec<Input>, Box<dyn Error>> {
        let mut vector: Vec<Input> = Vec::new();
        let mut countt: u32 = count;
        while countt > 0 {
            let input = Self::deserialize(payload);
            match input {
                Ok(a) => {
                    vector.push(a);
                    countt -= 1;
                }
                Err(_) => {
                    return Err("Failed to deserialize".into());
                }
            }
        }
        Ok(vector)
    }

    fn add_to_stack(
        stack: &mut Vec<u8>,
        i: u8,
        signature_script: &mut Vec<u8>,
        start_last_addition: &mut usize,
    ) -> Vec<u8> {
        let binding = signature_script.drain(..(i as usize)).collect::<Vec<u8>>();
        *start_last_addition = stack.len();
        stack.extend_from_slice(&binding);
        binding
    }

    pub fn get_sig_and_pubkey(&self) -> Vec<Vec<u8>> {
        let mut vector = vec![vec![]];
        let mut stack = Vec::new();
        let mut start_last_addition = 0;
        let mut clone_sript = self.signature_script.clone();
        while !clone_sript.is_empty() {
            let i = clone_sript.remove(0);
            if i < 76 && i < clone_sript.len() as u8 {
                vector.push(Self::add_to_stack(
                    &mut stack,
                    i,
                    &mut clone_sript,
                    &mut start_last_addition,
                ));
            }
        }
        vector
    }

    pub fn get_outpoint(&self) -> Outpoint {
        self.previous_tx
    }

    pub fn get_script(&self) -> Vec<u8> {
        self.signature_script.clone()
    }
    pub fn get_signature_script_length(&self) -> CompactSize {
        self.signature_script_length.clone()
    }
    pub fn get_sequence_number(&self) -> u32 {
        self.sequence_number
    }

    fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
        u32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }

    pub fn update_script(&mut self, script: Vec<u8>) {
        self.signature_script = script;
        self.signature_script_length =
            CompactSize::from_usize_to_compact_size(self.signature_script.len());
    }
}

impl CSVFormat for Input {
    fn get_csv_format(&self) -> Vec<String> {
        // Previous tx to string
        let previous_tx_to_string = self.previous_tx.get_csv_string();

        let signature_script_length_to_string = self.signature_script_length.get_csv_string();

        // Parse signature_script to string
        let mut signature_script_to_string_vec = Vec::new();
        for i in &self.signature_script {
            signature_script_to_string_vec.push(i.to_string());
        }

        let mut signature_script_to_string = signature_script_to_string_vec.join(",");
        signature_script_to_string = format!("[{}]", signature_script_to_string);

        // Sequence number to string
        let sequence_number_to_string = self.sequence_number.to_string();

        // Return vector
        vec![
            previous_tx_to_string,
            signature_script_length_to_string,
            signature_script_to_string,
            sequence_number_to_string,
        ]
    }
}

#[cfg(test)]

mod input_tests {
    use super::*;

    #[test]
    fn test_input_serialize() {
        let input = Input::new(
            Outpoint::new([0u8; 32], 0),
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            0,
        );
        let input_serialized = input.serialize();
        let input_serialized_expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        assert_eq!(input_serialized, input_serialized_expected);
    }

    #[test]
    fn test_input_size() {
        let input = Input::new(
            Outpoint::new([0u8; 32], 0),
            CompactSize {
                prefix: 0,
                number_vec: vec![14],
                number: 1,
            },
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            0,
        );
        let input_size = input.size();
        let input_size_expected = 55;
        assert_eq!(input_size, input_size_expected);
    }

    #[test]
    fn test_input_get_csv_format_outpoint() {
        let input = Input::new(
            Outpoint::new(
                [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                10,
            ),
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 3, 5, 7],
            9,
        );

        let input_csv_format = input.get_csv_format();
        assert_eq!(input_csv_format[0], "[12345668900000000000000000000000,10]");
    }

    #[test]
    fn test_input_get_csv_format_signature_script_length() {
        let input = Input::new(
            Outpoint::new(
                [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                10,
            ),
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 3, 5, 7],
            9,
        );

        let input_csv_format = input.get_csv_format();
        assert_eq!(input_csv_format[1], "[1,2]");
    }

    #[test]
    fn test_input_get_csv_format_signature_script() {
        let input = Input::new(
            Outpoint::new(
                [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                10,
            ),
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 3, 5, 7],
            9,
        );

        let input_csv_format = input.get_csv_format();
        assert_eq!(input_csv_format[2], "[1,3,5,7]");
    }

    #[test]
    fn test_input_get_csv_format_sequence_number() {
        let input = Input::new(
            Outpoint::new(
                [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ],
                10,
            ),
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 3, 5, 7],
            9,
        );

        let input_csv_format = input.get_csv_format();
        assert_eq!(input_csv_format[3], "9");
    }
}
