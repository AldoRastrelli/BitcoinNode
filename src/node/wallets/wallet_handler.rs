use super::wallet::Wallet;
use crate::{message_structs::tx_message::TXMessage, node::utxo_collector::UtxoCollector};
use std::{collections::HashMap, error::Error};

pub struct WalletHandler {
    pub wallets: Vec<Wallet>,
    actual_wallet: usize,
}

impl Default for WalletHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletHandler {
    /// Create a new WalletHandler
    pub fn new() -> WalletHandler {
        WalletHandler {
            wallets: vec![],
            actual_wallet: 0,
        }
    }

    /// Creates a WalletHandler from a vector of wallets produced by opening the file
    pub fn open(wallets: Vec<Wallet>) -> WalletHandler {
        WalletHandler {
            wallets,
            actual_wallet: 0,
        }
    }

    /// Create a new wallet as long as the data provided is correct
    pub fn new_wallet(&mut self, order: (String, String)) -> bool {
        if self.exist_wallet(order.0.clone()) {
            return false;
        }
        if order.0.is_empty() || order.1.is_empty() {
            return false;
        }
        self.actual_wallet = self.wallets.len();

        match Wallet::new(self.wallets.len(), order.0, order.1) {
            Some(wallet) => self.wallets.push(wallet),
            None => return false,
        }
        true
    }

    /// Returns positive if a wallet with that name already exists
    fn exist_wallet(&mut self, new_name: String) -> bool {
        for wallet in self.wallets.iter() {
            if wallet.get_name() == new_name {
                return true;
            }
        }
        false
    }

    /// Make a wallet switch
    pub fn switch_wallet(&mut self, new_wallet: String) {
        for wallet in self.wallets.iter() {
            if wallet.get_name() == new_wallet {
                self.actual_wallet = wallet.get_id();
                break;
            }
        }
    }

    /// Get all wallets names
    pub fn get_all_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for wallet in self.wallets.iter() {
            names.push(wallet.get_name());
        }
        names
    }

    /// Get all data from wallets
    pub fn get_all_data(&self) -> (Vec<String>, HashMap<String, String>) {
        (self.get_all_names(), self.actual_wallet_get_data())
    }

    pub fn get_actual_wallet(&self) -> Option<Wallet> {
        if self.wallets.is_empty() {
            return None;
        }
        Some(self.wallets[self.actual_wallet].clone())
    }

    /// Creates a hash matching for each label of the interface its corresponding data
    pub fn actual_wallet_get_data(&self) -> HashMap<String, String> {
        let mut hash = HashMap::new();
        let actual_wallet = &self.wallets[self.actual_wallet];
        hash.insert(
            "label_available".to_string(),
            actual_wallet.get_balance().to_string(),
        );
        hash.insert(
            "public_key_label".to_string(),
            actual_wallet.keys_handler.get_pubkey_string(),
        );
        hash.insert(
            "balance_send_fix".to_string(),
            actual_wallet.get_balance().to_string(),
        );
        hash.insert(
            "address_label".to_string(),
            actual_wallet.keys_handler.get_address(),
        );
        hash.insert(
            "label_available".to_string(),
            actual_wallet.get_balance().to_string(),
        );
        hash.insert(
            "balance_send_fix".to_string(),
            actual_wallet.get_balance().to_string(),
        );
        hash.insert(
            "label_inmature".to_string(),
            actual_wallet.get_pending().to_string(),
        );
        hash.insert(
            "label_total".to_string(),
            (actual_wallet.get_balance() as i32 + actual_wallet.get_pending()).to_string(),
        );

        hash
    }

    pub fn get_actual_balance(&self) -> u32 {
        self.wallets[self.actual_wallet].get_balance()
    }

    pub fn is_empty(&self) -> bool {
        self.wallets.len() == 0
    }

    pub fn create_transaction(
        &mut self,
        order: (String, String, i32, i32),
    ) -> Result<TXMessage, Box<dyn Error>> {
        let address = order.0;
        let amount = order.2;
        let fee = order.3;
        return self.wallets[self.actual_wallet].create_transaction(&address, amount, fee);
    }

    pub fn add_utxo_to_wallets(&mut self, tx_collector: &UtxoCollector) {
        // Add utxos to each of the wallets
        for wallet in self.wallets.iter_mut() {
            println!("aca entra");
            let addr = wallet.keys_handler.get_address().clone();

            let utxos = match tx_collector.utxos.get(&addr) {
                Some(utxos) => utxos,
                None => continue,
            };

            wallet.update_utxos(utxos);
        }
    }
}

#[cfg(test)]

mod wallet_handler_tests {
    use super::*;

    #[test]
    fn test_wallet_handler_new() {
        let wallet_handler = WalletHandler::new();
        assert!(wallet_handler.wallets.is_empty());
    }

    #[test]
    fn test_wallet_handler_new_wallet() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert_eq!(wallet_handler.wallets.len(), 1);
    }

    #[test]
    fn test_wallet_handler_new_wallet_with_empty_name() {
        let mut wallet_handler = WalletHandler::new();
        assert!(!wallet_handler.new_wallet((
            "".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert_eq!(wallet_handler.wallets.len(), 0);
    }

    #[test]
    fn test_wallet_handler_new_wallet_with_empty_key() {
        let mut wallet_handler = WalletHandler::new();
        assert!(!wallet_handler.new_wallet(("test_name".to_string(), "".to_string())));
        assert_eq!(wallet_handler.wallets.len(), 0);
    }

    #[test]
    fn test_wallet_handler_new_wallet_with_empty_name_and_key() {
        let mut wallet_handler = WalletHandler::new();
        assert!(!wallet_handler.new_wallet(("".to_string(), "".to_string())));
        assert_eq!(wallet_handler.wallets.len(), 0);
    }

    #[test]
    fn test_wallet_handler_new_wallet_with_same_name() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert!(!wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert_eq!(wallet_handler.wallets.len(), 1);
    }

    #[test]
    fn test_wallet_handler_get_all_names() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert!(wallet_handler.new_wallet((
            "test_name2".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert_eq!(
            wallet_handler.get_all_names(),
            vec!["test_name".to_string(), "test_name2".to_string()]
        );
    }

    #[test]
    fn test_wallet_handler_get_all_data() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));
        assert!(wallet_handler.new_wallet((
            "test_name2".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));

        let mut hash = HashMap::new();
        hash.insert("label_available".to_string(), "0".to_string());
        hash.insert(
            "public_key_label".to_string(),
            "03da2b61a2d639eac016bc256d5dafcd5e5bdb78b7cf87f0c459e865025254bb5a".to_string(),
        );
        hash.insert("balance_send_fix".to_string(), "0".to_string());
        hash.insert(
            "address_label".to_string(),
            "mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc".to_string(),
        );
        hash.insert("label_available".to_string(), "0".to_string());
        hash.insert("balance_send_fix".to_string(), "0".to_string());
        hash.insert("label_total".to_string(), "0".to_string());
        hash.insert("label_inmature".to_string(), "0".to_string());

        assert_eq!(
            wallet_handler.get_all_data(),
            (
                vec!["test_name".to_string(), "test_name2".to_string()],
                hash
            )
        );
    }

    #[test]
    fn test_actual_wallet_get_data() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));

        let mut hash = HashMap::new();
        hash.insert("label_available".to_string(), "0".to_string());
        hash.insert(
            "public_key_label".to_string(),
            "03da2b61a2d639eac016bc256d5dafcd5e5bdb78b7cf87f0c459e865025254bb5a".to_string(),
        );
        hash.insert("balance_send_fix".to_string(), "0".to_string());
        hash.insert(
            "address_label".to_string(),
            "mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc".to_string(),
        );
        hash.insert("label_available".to_string(), "0".to_string());
        hash.insert("balance_send_fix".to_string(), "0".to_string());
        hash.insert("label_total".to_string(), "0".to_string());
        hash.insert("label_inmature".to_string(), "0".to_string());
        assert_eq!(wallet_handler.actual_wallet_get_data(), hash);
    }

    #[test]
    fn test_wallet_handler_get_actual_balance() {
        let mut wallet_handler = WalletHandler::new();
        assert!(wallet_handler.new_wallet((
            "test_name".to_string(),
            "5032554e9d661af4e3fe58ef485231358925d39996830dac9eace8cadfbea9cd".to_string()
        )));

        assert_eq!(wallet_handler.get_actual_balance(), 0);
    }
}
