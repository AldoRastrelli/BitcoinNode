use super::common_traits::csv_format::CSVFormat;
use crate::message_structs::compact_size::CompactSize;
use std::vec;

#[derive(Debug, PartialEq, Clone)]
pub struct Output {
    pub value: i64,
    pub script_length: CompactSize,
    pub script: Vec<u8>,
}

impl Output {
    pub fn new(value: i64, script_length: CompactSize, script: Vec<u8>) -> Output {
        Self {
            value,
            script_length,
            script,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        output.extend_from_slice(&self.value.to_le_bytes());
        output.extend_from_slice(&self.script_length.serialize());
        for i in self.script.clone() {
            output.extend_from_slice(&[i]);
        }
        output
    }

    pub fn size(&self) -> u32 {
        let mut size = 44 + self.script_length.size();
        for _i in &self.script {
            size += 1;
        }
        size as u32
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Output {
        let value = Self::from_le_bytes_i64(payload);
        let script_length = CompactSize::deserialize(payload);
        let script = payload
            .drain(..(script_length.get_number()))
            .collect::<Vec<u8>>();

        Output::new(value, script_length, script)
    }

    pub fn deserialize_to_vec(payload: &mut Vec<u8>, count: u32) -> Vec<Output> {
        let mut vector: Vec<Output> = Vec::new();
        let mut countt: u32 = count;
        while countt > 0 {
            let output = Self::deserialize(payload);
            vector.push(output);
            countt -= 1;
        }
        vector
    }

    pub fn get_script(&self) -> Vec<u8> {
        self.script.clone()
    }

    pub fn get_value(&self) -> i64 {
        self.value
    }

    fn from_le_bytes_i64(payload: &mut Vec<u8>) -> i64 {
        i64::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }

    #[allow(dead_code)]
    fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
        u32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }
}

impl CSVFormat for Output {
    /// Example:
    /// Output::new(100, 2, vec![1, 2]);
    /// expected output: ["100", "2", "[1,2]"]
    fn get_csv_format(&self) -> Vec<String> {
        let value_to_string = self.value.to_string();

        // script_length_to_string
        let script_length_to_string = self.script_length.get_csv_string();

        // Parse script to string
        let mut script_to_string_vec = Vec::new();
        for i in &self.script {
            script_to_string_vec.push(i.to_string());
        }

        let mut script_to_string = script_to_string_vec.join(",");
        script_to_string = format!("[{}]", script_to_string);

        // Return vector
        vec![value_to_string, script_length_to_string, script_to_string]
    }
}

#[cfg(test)]

mod output_tests {
    use super::*;

    #[test]
    fn test_output_get_csv_format_value() {
        let output = Output::new(
            100,
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 2],
        );
        let output_csv = output.get_csv_format();
        assert_eq!(output_csv[0], "100");
    }

    #[test]
    fn test_output_get_csv_format_script_length() {
        let output = Output::new(
            100,
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 2],
        );
        let output_csv = output.get_csv_format();
        assert_eq!(output_csv[1], "[1,2]");
    }

    #[test]
    fn test_output_get_csv_format_script() {
        let output = Output::new(
            100,
            CompactSize {
                prefix: 1,
                number_vec: vec![1, 2],
                number: 3,
            },
            vec![1, 2],
        );
        let output_csv = output.get_csv_format();
        println!("{:?}", output_csv);
        assert_eq!(output_csv[2], "[1,2]");
    }
}
