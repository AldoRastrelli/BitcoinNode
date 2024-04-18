use crate::message_structs::compact_size::CompactSize;
#[derive(Debug, PartialEq)]
pub struct NetworkAddress2 {
    time: u32,
    services: CompactSize,
    network_id: u8,
    addres_size: CompactSize,
    address: Vec<u8>,
    port: u16,
}

impl NetworkAddress2 {
    pub fn new(
        time: u32,
        services: CompactSize,
        network_id: u8,
        addres_size: CompactSize,
        address: Vec<u8>,
        port: u16,
    ) -> NetworkAddress2 {
        Self {
            time,
            services,
            network_id,
            addres_size,
            address,
            port,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut network_addres2: Vec<u8> = Vec::new();
        network_addres2.extend_from_slice(&self.time.to_le_bytes());
        network_addres2.extend_from_slice(&self.services.serialize());
        network_addres2.extend_from_slice(&self.network_id.to_le_bytes());
        network_addres2.extend_from_slice(&self.addres_size.serialize());
        for i in &self.address {
            network_addres2.extend_from_slice(&i.to_be_bytes());
        }
        network_addres2.extend_from_slice(&self.port.to_be_bytes());
        network_addres2
    }

    pub fn size(&self) -> u32 {
        let size = 19 + &self.addres_size.size() + self.addres_size.get_number();
        size as u32
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Vec<NetworkAddress2> {
        let mut ip_addresses: Vec<NetworkAddress2> = Vec::new();
        let _x = 0;
        while !payload.is_empty() {
            let time = Self::from_le_bytes_u32(payload);
            let services = CompactSize::deserialize(payload);
            let network_id = u8::from_le_bytes([payload.remove(0)]);
            let addres_size = CompactSize::deserialize(payload);
            let address_serialize = payload
                .drain(..addres_size.get_number())
                .collect::<Vec<u8>>();
            let mut address: Vec<u8> = Vec::new();
            for i in address_serialize {
                address.push(u8::from_be_bytes([i]));
            }
            let port = u16::from_be_bytes([payload.remove(0), payload.remove(0)]);
            let network_addres =
                NetworkAddress2::new(time, services, network_id, addres_size, address, port);
            ip_addresses.push(network_addres);
        }
        ip_addresses
    }

    fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
        u32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }
}
