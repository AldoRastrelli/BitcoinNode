use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use std::io::Write;
use std::net::TcpStream;

use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::inv::Inv;

use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct InvOrGetDataMessage {
    count: CompactSize,
    inventory: Vec<Inv>,
}

impl InvOrGetDataMessage {
    pub fn new(count: CompactSize, inventory: Vec<Inv>) -> InvOrGetDataMessage {
        Self { count, inventory }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut inv_or_get_data: Vec<u8> = Vec::new();
        inv_or_get_data.extend_from_slice(&self.count.serialize());
        for i in &self.inventory {
            let inv_serialize = i.serialize();
            for j in inv_serialize {
                inv_or_get_data.extend_from_slice(&[j]);
            }
        }
        inv_or_get_data
    }

    pub fn send_inv(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_inv_message = self.serialize();
        let payload = self.count.size() + 36 * self.inventory.len();
        let header_inv_message = BitcoinMessageHeader::message(
            &serialize_inv_message,
            [
                b'i', b'n', b'v', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_inv_message.header(&serialize_inv_message);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn send_not_found(
        &self,
        mut stream: &TcpStream,
    ) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_inv_message = self.serialize();
        let payload = self.count.size() + 36 * self.inventory.len();
        let header_inv_message = BitcoinMessageHeader::message(
            &serialize_inv_message,
            [
                b'n', b'o', b't', b'f', b'o', b'u', b'n', b'd', 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_inv_message.header(&serialize_inv_message);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn send_get_data(
        &self,
        mut stream: &TcpStream,
    ) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_get_data = self.serialize();
        let payload = self.count.size() + 36 * self.inventory.len();
        let header_get_data = BitcoinMessageHeader::message(
            &serialize_get_data,
            [
                b'g', b'e', b't', b'd', b'a', b't', b'a', 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_get_data.header(&serialize_get_data);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<InvOrGetDataMessage, Box<dyn Error>> {
        let count = CompactSize::deserialize(payload);
        let inventory = match Inv::deserialize_to_vec(payload, count.get_number()) {
            Ok(inventory) => inventory,
            Err(e) => return Err(e),
        };

        Ok(InvOrGetDataMessage::new(count, inventory))
    }

    pub fn inv(&self)->Vec<Inv>{
        self.inventory.clone()
    }

    pub fn count(&self) -> usize {
        self.count.get_number()
    }

    pub fn ask_for_tx(&self, stream: &mut TcpStream) {
        let mut vector = vec![];
        for i in &self.inventory {
            if i.type_equal(1) {
                vector.push(*i)
            }
        }
        let get_data = InvOrGetDataMessage {
            count: CompactSize::from_usize_to_compact_size(vector.len()),
            inventory: vector,
        };
        let _result = get_data.send_get_data(stream);
        println!("enviado get_data: {:?}", get_data);
    }
}
