use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::utils::configs::config::get_protocol_version_i32;
use chrono::prelude::*;
use std::error::Error;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct VersionMessage {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    addr_recv_ip: [u8; 16],
    addr_recv_port: u16,
    addr_from_services: u64,
    addr_from_ip: [u8; 16],
    addr_from_port: u16,
    nonce: u64,
    user_agent: u8,
    start_height: i32,
    relay: u8,
}

impl VersionMessage {
    #[allow(dead_code)]
    fn build_default() -> Result<Self, Box<dyn Error>> {
        let now = Utc::now();
        let protocol_version = match get_protocol_version_i32() {
            Ok(protocol_version) => protocol_version,
            Err(e) => return Err(e),
        };

        Ok(VersionMessage {
            version: protocol_version,
            services: 1,
            timestamp: now.timestamp(),
            addr_recv_services: 0,
            addr_recv_ip: [0u8; 16],
            addr_recv_port: 0,
            addr_from_services: 0,
            addr_from_ip: [0u8; 16],
            addr_from_port: 0,
            nonce: 69_897_920_387_055_634, // rand, chronos
            user_agent: 0,
            start_height: 0,
            relay: 0,
        })
    }

    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut version_message: Vec<u8> = Vec::new();
        version_message.extend_from_slice(&self.version.to_le_bytes());
        version_message.extend_from_slice(&self.services.to_le_bytes());
        version_message.extend_from_slice(&self.timestamp.to_le_bytes());
        version_message.extend_from_slice(&self.addr_recv_services.to_le_bytes());
        version_message.extend_from_slice(&Self::array_serialize(&self.addr_recv_ip));
        version_message.extend_from_slice(&self.addr_recv_port.to_be_bytes());
        version_message.extend_from_slice(&self.addr_from_services.to_le_bytes());
        version_message.extend_from_slice(&Self::array_serialize(&self.addr_from_ip));
        version_message.extend_from_slice(&self.addr_from_port.to_be_bytes());
        version_message.extend_from_slice(&self.nonce.to_le_bytes());
        version_message.extend_from_slice(&self.user_agent.to_le_bytes());
        version_message.extend_from_slice(&self.start_height.to_le_bytes());
        version_message.extend_from_slice(&self.relay.to_le_bytes());
        version_message
    }

    /// # Errors
    /// This fails if write_all fails
    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_version = self.serialize();
        let payload = 86;
        let header_version = BitcoinMessageHeader::message(
            &serialize_version,
            [
                b'v', b'e', b'r', b's', b'i', b'o', b'n', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_version.header(&serialize_version);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    /// # Errors
    /// This fails if the payload is not the correct size
    pub fn deserialize(
        payload: &mut Vec<u8>,
    ) -> Result<VersionMessage, Box<dyn std::error::Error>> {
        let version = Self::from_le_bytes_i32(payload);

        let services = Self::from_le_bytes_u64(payload);

        let timestamp = Self::from_le_bytes_i64(payload);

        let addr_recv_services = Self::from_le_bytes_u64(payload);

        let addr_recv_ip = match payload.drain(0..16).collect::<Vec<u8>>().try_into() {
            Ok(a) => Self::array_deserialize(a),
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };

        let addr_recv_port = u16::from_be_bytes([payload.remove(0), payload.remove(0)]);

        let addr_from_services = Self::from_le_bytes_u64(payload);

        let addr_from_ip = match payload.drain(0..16).collect::<Vec<u8>>().try_into() {
            Ok(a) => Self::array_deserialize(a),
            Err(_) => {
                return Err("Failed to deserialize".into());
            }
        };

        let addr_from_port = u16::from_be_bytes([payload.remove(0), payload.remove(0)]);

        let nonce = Self::from_le_bytes_u64(payload);

        let user_agent = u8::from_le_bytes([payload.remove(0)]);

        let start_height = Self::from_le_bytes_i32(payload);

        let relay = u8::from_le_bytes([payload.remove(0)]);
        Ok(VersionMessage {
            version,
            services,
            timestamp,
            addr_recv_services,
            addr_recv_ip,
            addr_recv_port,
            addr_from_services,
            addr_from_ip,
            addr_from_port,
            nonce,
            user_agent,
            start_height,
            relay,
        })
    }

    fn array_serialize(array: &[u8; 16]) -> [u8; 16] {
        let mut array_new = [0u8; 16];
        for i in 0..16 {
            array_new[i] = array[i].to_be_bytes()[0];
        }
        array_new
    }

    fn array_deserialize(array: [u8; 16]) -> [u8; 16] {
        let mut array_new = [0u8; 16];
        for i in 0..16 {
            array_new[i] = u8::from_be_bytes([array[i]]);
        }
        array_new
    }

    fn from_le_bytes_u64(payload: &mut Vec<u8>) -> u64 {
        u64::from_le_bytes([
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

    fn from_le_bytes_i32(payload: &mut Vec<u8>) -> i32 {
        i32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }
}

#[cfg(test)]
mod version_message_test {
    use super::*;

    #[test]
    fn test_serialize() {
        let header = [
            127, 17, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 188, 143, 94, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 128, 53, 203, 201, 121, 83, 248,
            0, 0, 0, 0, 0, 0,
        ];
        let mut payload = header.to_vec();
        let version = VersionMessage::deserialize(&mut payload).unwrap();
        let bytes: Vec<u8> = vec![
            127, 17, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 188, 143, 94, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 128, 53, 203, 201, 121, 83, 248,
            0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(version.serialize(), bytes);
    }

    #[test]
    fn test_deserialize() {
        let header = [
            127, 17, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 188, 143, 94, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 128, 53, 203, 201, 121, 83, 248,
            0, 0, 0, 0, 0, 0,
        ];
        let mut payload = header.to_vec();
        let version = VersionMessage::deserialize(&mut payload).unwrap();
        let version2 = VersionMessage {
            version: 70015,
            services: 1,
            timestamp: 1415483324,
            addr_recv_services: 0,
            addr_recv_ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            addr_recv_port: 0,
            addr_from_services: 0,
            addr_from_ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            addr_from_port: 0,
            nonce: 17893779652077781010,
            user_agent: 0,
            start_height: 0,
            relay: 0,
        };
        assert_eq!(version, version2);
    }
}
