use std::{collections::HashMap, sync::MutexGuard};

use crate::{
    message_structs::{outpoint::Outpoint, output::Output, tx_message::TXMessage},
    utils::script_tools::{bitcoin_address_in_b58_input, bitcoin_address_in_b58_output},
};

#[derive(Debug)]
pub struct UtxoCollector {
    pub utxos: HashMap<String, Vec<(Outpoint, Output)>>,
}

impl Clone for UtxoCollector {
    fn clone(&self) -> Self {
        UtxoCollector {
            utxos: self.utxos.clone(),
        }
    }
}

impl Default for UtxoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl UtxoCollector {
    pub fn new() -> Self {
        UtxoCollector {
            utxos: HashMap::new(),
        }
    }

    // This could use a refactor
    pub fn add_utxo(&mut self, tx: TXMessage) {
        // Save all new outputs that come inside the tx
        for (index, output) in tx.get_output().iter().enumerate() {
            let bitcoin_address = bitcoin_address_in_b58_output(&output.get_script());
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.utxos.entry(bitcoin_address.clone())
            {
                let outpoint = Outpoint::new(tx.get_id(), 0);
                let values_push = (outpoint, output.clone());
                e.insert(vec![values_push]);
            //println!("address match")
            } else if let Some(x) = self.utxos.get_mut(&bitcoin_address) {
                let outpoint = Outpoint::new(tx.get_id(), index as u32);

                let values_push = (outpoint, output.clone());
                if !(x.contains(&values_push)) {
                    x.push(values_push);
                    //println!("address match")
                }
            }
        }

        // Foreach input, check if there's any outpoint that matches the ones we have. If so, remove it from utxos
        for input in tx.get_input() {
            let bitcoin_address = bitcoin_address_in_b58_input(&input.get_script());

            if self.utxos.contains_key(&bitcoin_address) {
                if let Some(x) = self.utxos.get_mut(&bitcoin_address) {
                    let outpoint = input.get_outpoint();
                    let index = match x.iter().position(|x| x.0 == outpoint) {
                        Some(x) => x,
                        None => continue,
                    };
                    x.remove(index);
                }
            }
        }

        for i in self.utxos.keys() {
            let input = tx.get_input();
            let outputs = tx.get_output();
            for j in outputs {
                if bitcoin_address_in_b58_output(&j.get_script()) == *i {
                    //println!("cuenta en transferencia: {:?}",i);
                }
            }
            for k in input {
                if bitcoin_address_in_b58_input(&k.get_script()) == *i {
                    //println!("cuenta en transferencia: {:?}",i);
                } else {
                    println!("{:?}", bitcoin_address_in_b58_input(&k.get_script()));
                }
            }
        }
    }

    pub fn get_utxos(&mut self) -> &mut HashMap<String, Vec<(Outpoint, Output)>> {
        &mut self.utxos
    }

    pub fn _get_utxos_for_address(&self, _address: &str) -> Vec<(Outpoint, Output)> {
        unimplemented!("get_utxos_for_address not implemented")
    }

    // // This could use a refactor
    pub fn create_address_utxo(
        &mut self,
        utxo_set: &mut MutexGuard<HashMap<[u8; 32], Vec<Output>>>,
    ) {
        // Clears the used utxos
        let utxo = self.utxos.clone();
        let keys = utxo.keys();
        for i in keys {
            if self.utxos.contains_key(i) {
                if let Some(x) = self.utxos.get_mut(i) {
                    x.clear();
                }
            }
        }

        // Save all the new utxos inside the Collector
        for (key, val) in utxo_set.iter() {
            for (num, i) in val.iter().enumerate() {
                let bitcoin_address = bitcoin_address_in_b58_output(&i.get_script());
                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.utxos.entry(bitcoin_address.clone())
                {
                    if !bitcoin_address.is_empty() {
                        let outpoint = Outpoint::new(*key, num as u32);
                        let values_push = (outpoint, i.clone());
                        e.insert(vec![values_push]);
                    }
                } else if let Some(x) = self.utxos.get_mut(&bitcoin_address) {
                    let outpoint = Outpoint::new(*key, num as u32);
                    let values_push = (outpoint, i.clone());
                    if !(x.contains(&values_push)) {
                        x.push(values_push);
                        //println!("address match:{:?}",bitcoin_address);
                    }
                }
            }
        }
    }
}
