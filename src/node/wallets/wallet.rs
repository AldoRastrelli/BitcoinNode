use crate::message_structs::{outpoint::Outpoint, tx_message::TXMessage};
use crate::utils::script_tools::from_adderss_to_vec;
use std::error::Error;

use super::{keys_handler::KeysHandler, transactions_handler::P2PKH};
use crate::message_structs::output::Output;

pub struct Wallet {
    id: usize,
    name: String,
    balance: u32,
    pending_balance: i32,
    pub keys_handler: KeysHandler,
    utxos: Vec<(Outpoint, Output)>, // Output for same address have same script, so there's no way to store the utxos with a unique id
}

impl Clone for Wallet {
    fn clone(&self) -> Self {
        Wallet {
            id: self.id,
            name: self.name.clone(),
            balance: self.balance,
            keys_handler: self.keys_handler.clone(),
            utxos: self.utxos.clone(),
            pending_balance: 0,
        }
    }
}

impl Wallet {
    /// Create a Wallet whenever the creation of a KeysHandler has been successful
    pub fn new(id: usize, name: String, private_key: String) -> Option<Wallet> {
        let keys_handler = match KeysHandler::new(&private_key) {
            Some(keys_handler) => keys_handler,
            None => return None,
        };
        Some(Wallet {
            id,
            name,
            balance: 0,
            keys_handler,
            utxos: Vec::new(),
            pending_balance: 0,
        })
    }

    /// Creates a Wallet with the data supplied from the files
    pub fn open(id: usize, name: String, private_key: String, balance: u32) -> Wallet {
        let keys_handler = match KeysHandler::new(&private_key) {
            Some(kh) => kh,
            None => panic!("Error al abrir wallet"),
        };

        Wallet {
            id,
            name,
            keys_handler,
            balance,
            utxos: Vec::new(),
            pending_balance: 0,
        }
    }

    /// Replace the actual value of the wallet's balance
    pub fn replace_balance(&mut self, income: u32) {
        println!("Replacing balance from {} to {}", self.balance, income);
        let outcome: i32 = self.balance as i32 - income as i32;
        if outcome <= self.pending_balance {
            self.pending_balance = 0;
        } else {
            self.pending_balance -= outcome;
        }
        self.balance = income;
    }

    pub fn reset_balance(&mut self) {
        self.balance = 0;
    }

    pub fn create_transaction(
        &mut self,
        address: &str,
        amount: i32,
        fee: i32,
    ) -> Result<TXMessage, Box<dyn Error>> {
        println!("wallet create transaction");
        if amount + fee > self.balance as i32 {
            return Err("wallet Not enough balance".into());
        }

        let address_vec = match from_adderss_to_vec(address) {
            Ok(address) => address.to_vec(),
            Err(_) => return Err("Error casting address to vec".into()),
        };

        println!("wallet address_vec");
        let (transaction, utxos_used): (TXMessage, Vec<(Outpoint, Output)>) =
            match P2PKH::create_transaction(
                &self.keys_handler,
                &self.utxos,
                &address_vec,
                amount as i64,
                fee as i64,
            ) {
                Ok((transaction, utxos_used)) => (transaction, utxos_used),
                Err(_) => return Err("Error creating transaction".into()),
            };

        println!("wallet create transaction P2PKH");
        for (x, (outpoint, utxo)) in utxos_used.iter().enumerate() {
            // Output for same address have same script, so there's no way to store the utxos with a unique id
            let index = utxos_used
                .iter()
                .position(|x| *x == (*outpoint, utxo.clone()))
                .unwrap();
            self.utxos.remove(index - x);
        }
        self.pending_balance -= amount + fee;

        println!("wallet before send {:?}", self.pending_balance);
        Ok(transaction)
    }

    pub fn update_utxos(&mut self, utxos: &[(Outpoint, Output)]) {
        self.utxos = utxos.to_owned();
        println!("actualiza balance {:?}", utxos);
        let mut income = 0;
        for (_, output) in self.utxos.clone() {
            income += output.value;
        }
        self.replace_balance(income as u32);
    }

    /// Getters --------------------------

    pub fn get_balance(&self) -> u32 {
        self.balance
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_pending(&self) -> i32 {
        self.pending_balance
    }

    pub fn get_all_data(&self) -> String {
        format!(
            "{},{},{},{}",
            self.id.clone(),
            self.name.clone(),
            self.keys_handler.get_privkey(),
            self.balance.clone()
        )
    }

    pub fn get_address(&self) -> String {
        self.keys_handler.get_address()
    }
}

#[cfg(test)]

mod wallet_tests {

    use super::*;

    #[test]
    fn test_wallet_new() {
        let wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        );
        assert!(wallet.is_some());
    }

    #[test]
    fn test_wallet_get_balance() {
        let wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        );
        assert_eq!(wallet.unwrap().get_balance(), 0);
    }

    #[test]
    fn test_wallet_get_name() {
        let wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        );
        assert_eq!(wallet.unwrap().get_name(), "test_name");
    }

    #[test]
    fn test_wallet_get_id() {
        let wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        );
        assert_eq!(wallet.unwrap().get_id(), 0);
    }

    //#[test]
    fn _test_wallet_get_all_data() {
        let wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        );
        assert_eq!(
            wallet.unwrap().get_all_data(),
            "0,test_name,[3, 218, 43, 97, 162, 214, 57, 234, 192, 22, 188, 37, 109, 93, 175, 205, 94, 91, 219, 120, 183, 207, 135, 240, 196, 89, 232, 101, 2, 82, 84, 187, 90],0"
        );
    }

    #[test]
    fn test_wallet_modify_balance() {
        let mut wallet = Wallet::new(
            0,
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string(),
        )
        .unwrap();
        wallet.replace_balance(10);
        assert_eq!(wallet.get_balance(), 10);
    }
}
