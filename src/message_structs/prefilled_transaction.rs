use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::tx_message::TXMessage;
use std::error::Error;

#[derive(Debug)]
pub struct PrefilledTransaction {
    index: CompactSize,
    tx: TXMessage,
}

impl PrefilledTransaction {
    pub fn new(index: CompactSize, tx: TXMessage) -> PrefilledTransaction {
        Self { index, tx }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut prefilled_transaction: Vec<u8> = Vec::new();
        prefilled_transaction.extend_from_slice(&self.index.serialize());
        let tx_serialize = self.tx.serialize();
        for i in tx_serialize {
            prefilled_transaction.extend_from_slice(&[i]);
        }
        prefilled_transaction
    }

    pub fn size(&self) -> u32 {
        self.index.size() as u32 + self.tx.size()
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<PrefilledTransaction, Box<dyn Error>> {
        let index = CompactSize::deserialize(payload);
        let tx = TXMessage::deserialize(payload);
        match tx {
            Ok(tx) => Ok(PrefilledTransaction::new(index, tx)),
            Err(e) => Err(e),
        }
    }

    /// # Errors
    /// This function will return an error if it fails to deserialize the vector
    pub fn deserialize_to_vec(
        payload: &mut Vec<u8>,
        count: u32,
    ) -> Result<Vec<PrefilledTransaction>, Box<dyn Error>> {
        let mut vector: Vec<PrefilledTransaction> = Vec::new();
        let mut countt: u32 = count;
        while countt > 0 {
            let prefilled_transaction = Self::deserialize(payload);
            match prefilled_transaction {
                Ok(prefilled_transaction) => {
                    vector.push(prefilled_transaction);
                    countt -= 1;
                }
                Err(_e) => {
                    return Err("Failed to deserialize".into());
                }
            };
        }
        Ok(vector)
    }
}
