use std::error::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Inv {
    inv_type: u32,
    hash: [u8; 32],
}

impl Inv {
    pub fn new(inv_type: u32, hash: [u8; 32]) -> Inv {
        Self { inv_type, hash }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut inv: Vec<u8> = Vec::new();
        inv.extend_from_slice(&self.inv_type.to_le_bytes());
        inv.extend_from_slice(&self.hash);
        inv
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<Inv, Box<dyn Error>> {
        let inv_type = Self::from_le_bytes_u32(payload);

        let hash: [u8; 32] = match payload.drain(0..32).collect::<Vec<u8>>().try_into() {
            Ok(a) => a,
            Err(_) => {
                return Err("failed to deserialize".into());
            }
        };

        Ok(Inv::new(inv_type, hash))
    }

    pub fn deserialize_to_vec(
        payload: &mut Vec<u8>,
        count: usize,
    ) -> Result<Vec<Inv>, Box<dyn Error>> {
        let mut vector: Vec<Inv> = Vec::new();
        for _i in 0..count {
            match Self::deserialize(payload) {
                Ok(a) => {
                    vector.push(a);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(vector)
    }

    pub fn type_equal(self, n: u32) -> bool {
        self.inv_type == n
    }

    pub fn inv_type(&self)->u32{
        self.inv_type
    }

    pub fn hash(&self)->[u8;32]{
        self.hash
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
