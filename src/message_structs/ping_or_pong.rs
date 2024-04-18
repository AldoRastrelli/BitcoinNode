use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct PingOrPong {
    pub nonce: u64,
}

impl PingOrPong {
    pub fn new(nonce: u64) -> PingOrPong {
        Self { nonce }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut ping_or_pong: Vec<u8> = Vec::new();
        ping_or_pong.extend_from_slice(&self.nonce.to_le_bytes());
        ping_or_pong
    }

    pub fn send_ping(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_ping = self.serialize();
        let payload = 8;
        let header_ping = BitcoinMessageHeader::message(
            &serialize_ping,
            [
                b'p', b'i', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_ping.header(&serialize_ping);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn send_pong(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_pong = self.serialize();
        let payload = 8;
        let header_pong = BitcoinMessageHeader::message(
            &serialize_pong,
            [
                b'p', b'o', b'n', b'g', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_pong.header(&serialize_pong);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }
    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> PingOrPong {
        let nonce = Self::from_le_bytes_u64(payload);

        PingOrPong::new(nonce)
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
}
