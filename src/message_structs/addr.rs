use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::network_address::NetworkAddress;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub struct Addr {
    count: CompactSize,
    ip_addresses: Vec<NetworkAddress>,
}

impl Addr {
    pub fn new(count: CompactSize, ip_addresses: Vec<NetworkAddress>) -> Self {
        Self {
            count,
            ip_addresses,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut addr: Vec<u8> = Vec::new();
        addr.extend_from_slice(&self.count.serialize());
        for i in &self.ip_addresses {
            let ip_address_serialize = i.serialize();
            addr.extend_from_slice(&ip_address_serialize);
        }
        addr
    }

    pub fn send(&self, stream: &mut TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_addr = self.serialize();
        let payload = self.count.size() + 30 * self.count.get_number();
        let header_addr = BitcoinMessageHeader::message(
            &serialize_addr,
            [
                b'a', b'd', b'd', b'r', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_addr.header(&serialize_addr);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let count_deserialize = CompactSize::deserialize(payload);
        let ip_address_deserialize = NetworkAddress::deserialize(payload);

        Ok(Addr {
            count: count_deserialize,
            ip_addresses: ip_address_deserialize,
        })
    }
}
