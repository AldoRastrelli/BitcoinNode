use super::common_traits::csv_format::CSVFormat;
use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::input::Input;
use crate::message_structs::output::Output;
use crate::utils::array_tools::{from_le_bytes_i32, from_le_bytes_u32};
use bitcoin_hashes::{sha256, sha256d, Hash};
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq, Clone)]
pub struct TXMessage {
    version: i32,
    input_count: CompactSize,
    pub input_list: Vec<Input>,
    output_count: CompactSize,
    output_list: Vec<Output>,
    pub time: u32,
}

impl TXMessage {
    pub fn new(
        version: i32,
        input_count: CompactSize,
        input_list: Vec<Input>,
        output_count: CompactSize,
        output_list: Vec<Output>,
        time: u32,
    ) -> TXMessage {
        Self {
            version,
            input_count,
            input_list,
            output_count,
            output_list,
            time,
        }
    }

    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut tx_message: Vec<u8> = Vec::new();
        tx_message.extend_from_slice(&self.version.to_le_bytes());
        tx_message.extend_from_slice(&self.input_count.serialize());
        for i in &self.input_list {
            let serialize_input = i.serialize();
            for j in serialize_input {
                tx_message.extend_from_slice(&[j]);
            }
        }
        tx_message.extend_from_slice(&self.output_count.serialize());
        for i in &self.output_list {
            let serialize_output = i.serialize();
            for j in serialize_output {
                tx_message.extend_from_slice(&[j]);
            }
        }
        tx_message.extend_from_slice(&self.time.to_le_bytes());
        tx_message
    }

    #[must_use]
    pub fn size(&self) -> u32 {
        let mut size = 16 + self.input_count.size() as u32 + self.output_count.size() as u32;
        for i in &self.input_list {
            size += i.size();
        }
        for i in &self.output_list {
            size += i.size();
        }
        size
    }

    /// # Errors
    /// Returns an error if the message could not be sent
    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_tx = self.serialize();
        let payload = serialize_tx.len() as u32;
        let header_tx = BitcoinMessageHeader::message(
            &serialize_tx,
            [
                b't', b'x', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload,
        );
        let header = header_tx.header(&serialize_tx);
        stream.write_all(&header)?;
        println!("mensaje enviado correctamente (transaccion)");
        Ok("mensaje enviado correctamente (transaccion)")
    }

    /// # Errors
    /// Returns an error if the payload could not be deserialized
    pub fn deserialize(payload: &mut Vec<u8>) -> Result<TXMessage, Box<dyn std::error::Error>> {
        //payload.remove(0);
        let version = from_le_bytes_i32(payload);
        let input_count = CompactSize::deserialize(payload);
        let input_list = match Input::deserialize_to_vec(payload, input_count.get_number() as u32) {
            Ok(input_list) => input_list,
            Err(e) => return Err(e),
        };

        let output_count = CompactSize::deserialize(payload);
        let output_list = Output::deserialize_to_vec(payload, output_count.get_number() as u32);
        let time = from_le_bytes_u32(payload);

        Ok(TXMessage {
            version,
            input_count,
            input_list,
            output_count,
            output_list,
            time,
        })
    }

    /// # Errors
    /// Returns an error if the payload could not be deserialized
    pub fn deserialize_to_vec(
        payload: &mut Vec<u8>,
        mut count: usize,
    ) -> Result<Vec<TXMessage>, Box<dyn std::error::Error>> {
        let mut vector: Vec<TXMessage> = Vec::new();
        while count > 0 {
            let tx_message = Self::deserialize(payload);
            match tx_message {
                Ok(tx_message) => vector.push(tx_message),
                Err(e) => return Err(e),
            };
            count -= 1;
        }
        Ok(vector)
    }

    pub fn get_output(&self) -> Vec<Output> {
        let mut vector: Vec<Output> = vec![];
        for i in &self.output_list {
            let output = Output::deserialize(&mut i.serialize());
            vector.push(output);
        }
        vector
    }

    pub fn get_output_amounts(&self) -> Vec<i64> {
        let mut amounts = Vec::new();
        for output in &self.output_list {
            amounts.push(output.value);
        }
        amounts
    }

    pub fn get_input(&self) -> Vec<Input> {
        let mut vector: Vec<Input> = vec![];
        for i in &self.input_list {
            let Ok(input) =Input::deserialize(&mut i.serialize())else {panic!("problemas con el input")};
            vector.push(input);
        }
        vector
    }

    pub fn get_id(&self) -> [u8; 32] {
        let vector = Self::serialize(self);
        let hash = sha256::Hash::hash(&vector);
        //println!("hash previo a hash:{:?}",hash.to_byte_array());
        let hash2 = sha256::Hash::hash(&hash.to_byte_array());
        hash2.to_byte_array()
    }

    pub fn update_empty_script(&mut self, new_script: Vec<u8>) {
        for i in self.input_list.iter_mut() {
            if i.get_script().is_empty() {
                i.update_script(new_script);
                return;
            }
        }
    }

    pub fn sig_hash(&self, index: u32) -> Vec<u8> {
        let mut tx_message: Vec<u8> = Vec::new();
        tx_message.extend_from_slice(&self.version.to_le_bytes());
        tx_message.extend_from_slice(&self.input_count.serialize());
        for (n, i) in self.input_list.iter().enumerate() {
            let signature_len;
            let script;
            if n as u32 == index {
                signature_len = i.get_signature_script_length();
                script = i.get_script();
            } else {
                signature_len = CompactSize::from_usize_to_compact_size(0);
                script = vec![];
            }
            tx_message.extend_from_slice(
                &Input::new(
                    i.get_outpoint(),
                    signature_len,
                    script,
                    i.get_sequence_number(),
                )
                .serialize(),
            )
        }
        tx_message.extend_from_slice(&self.output_count.serialize());
        for i in &self.output_list {
            tx_message.extend_from_slice(&i.serialize());
        }
        tx_message.extend_from_slice(&self.time.to_le_bytes());
        tx_message.extend_from_slice(&1_u32.to_le_bytes());
        println!("before hash: {:?}", tx_message.to_vec());
        let h256 = sha256d::Hash::hash(&tx_message);
        let vector = h256.to_byte_array();
        println!("ready to sign: {:?}", vector.to_vec());
        vector.to_vec()
    }
}

impl CSVFormat for TXMessage {
    fn get_csv_format(&self) -> Vec<String> {
        // version and input count
        let version_to_string = self.version.to_string();

        // input count
        let input_count_to_string = self.input_count.get_csv_string();

        // input list
        let mut input_list_to_str_vec = Vec::new();
        for input in &self.input_list {
            let input_to_str = input.get_csv_format();
            for i in input_to_str {
                input_list_to_str_vec.push(i);
            }
        }

        input_list_to_str_vec.join(",");
        let input_list_to_str = format!("[{}]", input_list_to_str_vec.join(","));

        // output count
        let output_count_to_string = self.output_count.get_csv_string();

        // output list
        let mut output_list_to_str_vec = Vec::new();
        for output in &self.output_list {
            // For each output, we join the items with ',' and then we add '[' and ']' to the string to isolate single outputs from each other
            let output_vec = output.get_csv_format();

            let mut output_to_string = output_vec.join(",");
            output_to_string = format!("[{}]", output_to_string);
            // Add the output to the final vector
            output_list_to_str_vec.push(output_to_string);
        }

        // Join every outut with ',' and then add '[' and ']' to represent a list of outputs
        let output_list_to_str = format!("[{}]", output_list_to_str_vec.join(","));

        // time
        let time = self.time.to_string();

        // Return vector
        vec![
            version_to_string,
            input_count_to_string,
            input_list_to_str,
            output_count_to_string,
            output_list_to_str,
            time,
        ]
    }
}

#[cfg(test)]

mod tx_message_tests {
    use super::*;
    use crate::message_structs::outpoint::Outpoint;

    #[test]
    fn test_get_csv_format() {
        let hash = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x06, 0x08, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let outpoint = Outpoint::new(hash, 10);

        let input = Input::new(
            outpoint,
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 3, 5, 7],
            9,
        );

        let output1 = Output::new(
            100,
            CompactSize {
                prefix: 0x02,
                number: 0x02,
                number_vec: vec![0x02],
            },
            vec![1, 2],
        );
        let output2 = Output::new(
            200,
            CompactSize {
                prefix: 0x02,
                number: 0x02,
                number_vec: vec![0x02],
            },
            vec![3, 5],
        );

        let tx_message = TXMessage {
            version: 0x03,
            input_count: CompactSize {
                prefix: 0x02,
                number: 0x01,
                number_vec: vec![0x02],
            },
            input_list: vec![input],
            output_count: CompactSize {
                prefix: 0x02,
                number: 0x02,
                number_vec: vec![0x02],
            },
            output_list: vec![output1, output2],
            time: 1433835532,
        };

        let csv_format = tx_message.get_csv_format();
        let expected_csv_format = [
            "3",
            "[2]",
            "[[12345668900000000000000000000000,10],[1,2],[1,3,5,7],9]",
            "[2]",
            "[[100,[2],[1,2]],[200,[2],[3,5]]]",
            "1433835532",
        ];

        assert_eq!(csv_format, expected_csv_format);
    }
}
