#[derive(Debug, PartialEq)]
pub struct NetworkAddress {
    time: u32,
    services: u64,
    ip_address: [u8; 16],
    port: u16,
}

impl NetworkAddress {
    pub fn new(time: u32, services: u64, ip_address: [u8; 16], port: u16) -> NetworkAddress {
        Self {
            time,
            services,
            ip_address,
            port,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut network_addres: Vec<u8> = Vec::new();
        network_addres.extend_from_slice(&self.time.to_le_bytes());
        network_addres.extend_from_slice(&self.services.to_le_bytes());
        network_addres.extend_from_slice(&Self::array_serialize(&self.ip_address));
        network_addres.extend_from_slice(&self.port.to_be_bytes());
        network_addres
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Vec<NetworkAddress> {
        let mut ip_addresses: Vec<NetworkAddress> = Vec::new();
        for _i in 0..payload.len() / 30 {
            let network_addres = NetworkAddress {
                time: Self::from_le_bytes_u32(payload),
                services: Self::from_le_bytes_u64(payload),
                ip_address: match payload.drain(0..16).collect::<Vec<u8>>().try_into() {
                    Ok(a) => Self::array_deserialize(a),
                    Err(_e) => panic!("Failed to convert vector to array"),
                },
                port: u16::from_be_bytes([payload.remove(0), payload.remove(0)]),
            };
            ip_addresses.push(network_addres);
        }

        ip_addresses
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

    fn from_le_bytes_u32(payload: &mut Vec<u8>) -> u32 {
        u32::from_le_bytes([
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
            payload.remove(0),
        ])
    }
}
