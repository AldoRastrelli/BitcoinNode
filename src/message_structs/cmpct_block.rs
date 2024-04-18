use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::prefilled_transaction::PrefilledTransaction;
use crate::message_structs::block_message::BlockMessage;
use bitcoin_hashes::siphash24;
use crate::node::validation_engine::hashes::{header_calculate_doublehash_array_be,vec_calculate_simple_hash_array_le};
use rand::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

#[derive(Debug)]
pub struct CmpctBlock {
    block_header: BlockHeader,
    nonce: u64,
    shortids_length: CompactSize,
    shortids: Vec<Vec<u8>>,
    prefilled_txn_length: CompactSize,
    prefilled_txn: Vec<PrefilledTransaction>,
}

impl CmpctBlock {
    pub fn new(
        block_header: BlockHeader,
        nonce: u64,
        shortids_length: CompactSize,
        shortids: Vec<Vec<u8>>,
        prefilled_txn_length: CompactSize,
        prefilled_txn: Vec<PrefilledTransaction>,
    ) -> CmpctBlock {
        Self {
            block_header,
            nonce,
            shortids_length,
            shortids,
            prefilled_txn_length,
            prefilled_txn,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut cmpct_block: Vec<u8> = Vec::new();
        let block_header = self.block_header.serialize();
        for i in block_header {
            cmpct_block.extend_from_slice(&[i]);
        }
        cmpct_block.extend_from_slice(&self.nonce.to_le_bytes());
        cmpct_block.extend_from_slice(&self.shortids_length.serialize());
        for i in &self.shortids {
            cmpct_block.extend_from_slice(i);
        }
        cmpct_block.extend_from_slice(&self.prefilled_txn_length.serialize());
        for i in &self.prefilled_txn {
            let serialize = i.serialize();
            for j in serialize {
                cmpct_block.extend_from_slice(&[j]);
            }
        }
        cmpct_block
    }

    pub fn send(&self, mut stream: &TcpStream) -> Result<&str, Box<dyn std::error::Error>> {
        let serialize_cmpct_block = self.serialize();
        let payload = serialize_cmpct_block.len();
        let header_cmpct_block = BitcoinMessageHeader::message(
            &serialize_cmpct_block,
            [
                b'c', b'm', b'p', b'c', b't', b'b', b'l', b'o', b'c', b'k', 0x00, 0x00,
            ],
            payload as u32,
        );
        let header = header_cmpct_block.header(&serialize_cmpct_block);
        stream.write_all(&header)?;
        Ok("mensaje enviado correctamente")
    }

    pub fn deserialize(payload: &mut Vec<u8>) -> Result<CmpctBlock, Box<dyn std::error::Error>> {
        let block_header = match BlockHeader::deserialize(payload) {
            Ok(block_header) => block_header,
            Err(e) => return Err(e),
        };

        let nonce = Self::from_le_bytes_u64(payload);
        let shortids_length = CompactSize::deserialize(payload);
        let mut shortids: Vec<Vec<u8>> = Vec::new();
        for _i in 0..shortids_length.get_number() {
            let shortids_serialize = payload
            .drain(..8)
            .collect::<Vec<u8>>();
            shortids.push(shortids_serialize);
        }
        let prefilled_txn_length = CompactSize::deserialize(payload);
        let prefilled_txn = match PrefilledTransaction::deserialize_to_vec(
            payload,
            prefilled_txn_length.get_number() as u32,
        ) {
            Ok(prefilled_txn) => prefilled_txn,
            Err(e) => return Err(e),
        };

        Ok(CmpctBlock::new(
            block_header,
            nonce,
            shortids_length,
            shortids,
            prefilled_txn_length,
            prefilled_txn,
        ))
    }

    pub fn create_cmpt_block(block:BlockMessage,txs:HashMap<[u8;32],Vec<u8>>)->CmpctBlock{
        let block_header = block.block_header.clone();
        let mut rng = rand::thread_rng();
        let nonce: u64=rng.gen() ;//numero
        let mut hash_for_id =vec![];
        hash_for_id.extend_from_slice(&header_calculate_doublehash_array_be(&block_header).unwrap());
        hash_for_id.extend_from_slice(&nonce.to_le_bytes());
        let mut sha = vec_calculate_simple_hash_array_le(&hash_for_id).unwrap_or([0u8;32]).to_vec();
        let k0 = Self::from_le_bytes_u64(&mut sha);
        let k1 = Self::from_le_bytes_u64(&mut sha);
        let mut shortids = vec![];
        let mut prefilled_txn = vec![];
        for (i,tx) in block.get_tx().iter().enumerate(){
            if txs.contains_key(&tx.get_id()){
                let mut n=siphash24::Hash::hash_to_u64_with_keys(k0, k1, &tx.get_id()).to_be_bytes().to_vec();//crear el id con el hash y el nonce
                n.remove(0);
                n.remove(0);
                n.push(0);
                n.push(0);
                shortids.push(n);
            }
            else{
                prefilled_txn.push(PrefilledTransaction::new(CompactSize::from_usize_to_compact_size(i+1),tx.clone() ));
            }
        }
        CmpctBlock { block_header,nonce, shortids_length: CompactSize::from_usize_to_compact_size(shortids.len()), shortids , prefilled_txn_length: CompactSize::from_usize_to_compact_size(prefilled_txn.len()), prefilled_txn }
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
