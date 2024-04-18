use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::network_address2::NetworkAddress2;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct Addr2 {
    count: CompactSize,
    ip_addresses: Vec<NetworkAddress2>,
}

impl Addr2 {
    pub fn new(count: CompactSize, ip_addresses: Vec<NetworkAddress2>) -> Addr2 {
        Self {
            count,
            ip_addresses,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut addr2: Vec<u8> = Vec::new();
        addr2.extend_from_slice(&self.count.serialize());
        for i in &self.ip_addresses {
            let ip_address_serialize = i.serialize();
            for j in ip_address_serialize {
                addr2.extend_from_slice(&[j]);
            }
        }
        addr2
    }
    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_addr2 = self.serialize();
        let mut payload = self.count.size();
        for i in &self.ip_addresses {
            payload += i.size() as usize;
        }
        let header_addr2 = BitcoinMessageHeader::message(
            &serialize_addr2,
            [
                b'a', b'd', b'd', b'r', b'2', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_addr2.header(&serialize_addr2);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Addr2 {
        let count_deserialize = CompactSize::deserialize(payload);

        let ip_address_deserialize = NetworkAddress2::deserialize(payload);

        Addr2 {
            count: count_deserialize,
            ip_addresses: ip_address_deserialize,
        }
    }
}
