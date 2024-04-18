use bitcoin_hashes::{sha256d, Hash};
use std::io::Write;
use std::net::TcpStream;

use crate::utils::commands::{get_type, MessageType};

#[derive(Debug, PartialEq)]
pub struct BitcoinMessageHeader {
    magic: [u8; 4],
    command: [u8; 12],
    payload_size: u32,
    checksum: [u8; 4],
}

impl BitcoinMessageHeader {
    pub fn copy(&self) -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: self.magic,
            command: self.command,
            payload_size: self.payload_size,
            checksum: self.checksum,
        }
    }

    pub fn message(
        serialize_message: &[u8],
        command_message: [u8; 12],
        payload: u32,
    ) -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: command_message,
            payload_size: payload,
            checksum: Self::calculate_checksum(serialize_message),
        }
    }

    pub fn verack() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn empty_headers() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn mempool() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'm', b'e', b'm', b'p', b'o', b'o', b'l', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn get_addr() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'g', b'e', b't', b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn filter_clear() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'f', b'i', b'l', b't', b'e', b'r', b'c', b'c', b'e', b'a', b'r', 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn send_headers() -> BitcoinMessageHeader {
        BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b's', b'e', b'n', b'd', b'h', b'e', b'a', b'd', b'e', b'r', b's', 0x00,
            ],
            payload_size: 0,
            // This message has no payload. The checksum does not change.
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut vector: Vec<u8> = Vec::new();
        vector.extend_from_slice(&self.magic);
        vector.extend_from_slice(&self.command);
        vector.extend_from_slice(&self.payload_size.to_le_bytes());
        vector.extend_from_slice(&self.checksum);
        vector
    }

    pub fn header(&self, serialize_message: &[u8]) -> Vec<u8> {
        let mut header: Vec<u8> = Vec::new();
        _ = &header.extend_from_slice(&self.magic);
        _ = &header.extend_from_slice(&self.command);
        _ = &header.extend_from_slice(&self.payload_size.to_le_bytes());
        _ = &header.extend_from_slice(&self.checksum);
        _ = &header.extend_from_slice(serialize_message);
        header
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let mut header: Vec<u8> = Vec::new();
        _ = &header.extend_from_slice(&self.magic);
        _ = &header.extend_from_slice(&self.command);
        _ = &header.extend_from_slice(&self.payload_size.to_le_bytes());
        _ = &header.extend_from_slice(&self.checksum);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    fn validate_header(serialize_message: Vec<u8>) -> bool {
        if serialize_message.len() < 24 {
            return false;
        }
        let _magic = &serialize_message[0..4];
        let command = &serialize_message[4..16];
        for byte in command {
            if !byte.is_ascii() {
                return false;
            }
        }
        let payload = &serialize_message[17..20];
        for byte in payload {
            if !matches!(byte, _) {
                return false;
            }
        }
        let _checksum = &serialize_message[20..24];
        true
    }

    pub fn deserialize(
        serialize_message: &mut Vec<u8>,
    ) -> Result<(BitcoinMessageHeader, &mut Vec<u8>), Box<dyn std::error::Error>> {
        if !Self::validate_header(serialize_message.clone()) {
            return Err("Error in header received".into());
        }
        let message = BitcoinMessageHeader {
            magic: [
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
            ],
            command: [
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
            ],
            payload_size: u32::from_le_bytes([
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
            ]),
            checksum: [
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
                serialize_message.remove(0),
            ],
        };
        Ok((message, serialize_message))
    }

    pub fn command(&self) -> [u8; 12] {
        self.command
    }

    pub fn is_header_message(&self) -> u8 {
        match get_type(&self.command) {
            MessageType::HeadersMessage => 0,
            _ => 1,
        }
    }

    pub fn payload(&self) -> u32 {
        self.payload_size
    }

    fn calculate_checksum(payload: &[u8]) -> [u8; 4] {
        let hash = sha256d::Hash::hash(payload);
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash[..4]);
        checksum
    }

    pub fn get_csv_format(&self) -> Vec<String> {
        let magic_to_string = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            self.magic[0], self.magic[1], self.magic[2], self.magic[3]
        );

        let command_to_str = String::from_utf8_lossy(&self.command).to_string();

        let payload_size_to_string = format!("{:?}", self.payload_size);

        let checksum_to_string = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            self.checksum[0], self.checksum[1], self.checksum[2], self.checksum[3]
        );

        vec![
            magic_to_string,
            command_to_str,
            payload_size_to_string,
            checksum_to_string,
        ]
    }
}

#[cfg(test)]

mod bitcoin_message_headers_tests {
    use super::*;

    #[test]
    fn test_get_csv_format_transforms_magic_ok() {
        let message = BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        };

        let csv_format = message.get_csv_format();

        assert_eq!(csv_format[0], "0b110907");
    }

    #[test]
    fn test_get_csv_format_transforms_verack_ok() {
        let message = BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        };

        let csv_format = message.get_csv_format();

        assert_eq!(csv_format[1], "verack\0\0\0\0\0\0");
    }

    #[test]
    fn test_get_csv_format_transforms_payload_size_ok() {
        let message = BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        };

        let csv_format = message.get_csv_format();

        assert_eq!(csv_format[2], "0");
    }

    #[test]
    fn test_get_csv_format_transforms_checksum_ok() {
        let message = BitcoinMessageHeader {
            magic: [0x0b, 0x11, 0x09, 0x07],
            command: [
                b'v', b'e', b'r', b'a', b'c', b'k', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload_size: 0,
            checksum: [0x5d, 0xf6, 0xe0, 0xe2],
        };

        let csv_format = message.get_csv_format();

        assert_eq!(csv_format[3], "5df6e0e2");
    }
}
