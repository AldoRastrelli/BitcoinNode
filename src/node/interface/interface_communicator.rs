use glib::Sender;

use crate::{
    message_structs::{
        block_headers::BlockHeader, block_message::BlockMessage, tx_message::TXMessage,
    },
    node::{
        connection_manager::peers_connection::writer, utxo_collector::UtxoCollector,
        wallets::wallet_handler::WalletHandler,
    },
};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::node::validation_engine::merkles::merkle_tree::MerkleTree;
use crate::node::wallets::wallet::Wallet;

/// Communication channels node - interface
pub enum InterfaceMessages {
    DebugHeaders(BlockHeader),
    DebugBlocks(i32, BlockMessage, usize),
    AllTransactions(bool, TXMessage, bool),
    MyTransactions(TXMessage),
    SendTransaction((String, String, i32, i32)),
    AddWalletOrder((String, String)),
    WalletName(String),
    WalletSwitch(String),
    ActualWallet(HashMap<String, String>),
    InclusionProof(Vec<String>, Vec<String>),
    InclusionProofResult(bool),
    Close(()),
    Open((Vec<String>, HashMap<String, String>)),
}

pub struct InterfaceCommunicator {
    pub wallet_handler: Arc<Mutex<WalletHandler>>,
    opened: bool,
    pub blocks: Arc<Mutex<HashMap<String, BlockMessage>>>,
    pub transactions: Arc<Mutex<HashMap<String, TXMessage>>>,
}

impl Clone for InterfaceCommunicator {
    fn clone(&self) -> Self {
        InterfaceCommunicator {
            wallet_handler: self.wallet_handler.clone(),
            opened: self.opened,
            blocks: self.blocks.clone(),
            transactions: self.transactions.clone(),
        }
    }
}

impl Default for InterfaceCommunicator {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceCommunicator {
    /// Creates the InterfaceCommunicator object, if the wallets file is empty it creates one from zero, otherwise it creates it with the saved data
    pub fn new() -> InterfaceCommunicator {
        let mut opened = false;
        let wallets = match InterfaceCommunicator::open() {
            Some(wallets) => {
                opened = true;
                wallets
            }
            None => WalletHandler::new(),
        };
        InterfaceCommunicator {
            wallet_handler: Arc::new(Mutex::new(wallets)),
            opened,
            blocks: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Matches the channels and depending on which one it is, sends the necessary command to be executed
    pub fn start(
        &mut self,
        peers: Vec<String>,
        sender_to_interface: Sender<InterfaceMessages>,
        receiver_from_interface: Receiver<InterfaceMessages>,
    ) -> JoinHandle<()> {
        if self.opened {
            self.open_interface(sender_to_interface.clone());
        }
        let wallet_handler = Arc::clone(&self.wallet_handler);
        let blocks = Arc::clone(&self.blocks);
        let transactions = Arc::clone(&self.transactions);

        thread::spawn(move || {
            for message in receiver_from_interface {
                let sender_to_interface = sender_to_interface.clone();
                let peers = peers.clone();
                let wallet_handler = Arc::clone(&wallet_handler);
                let blocks = Arc::clone(&blocks);
                let transactions = Arc::clone(&transactions);
                Self::handle_messages(
                    message,
                    sender_to_interface,
                    peers,
                    wallet_handler,
                    blocks,
                    transactions,
                );
            }
        })
    }

    fn handle_messages(
        message: InterfaceMessages,
        sender_to_interface: Sender<InterfaceMessages>,
        peers: Vec<String>,
        wallet_handler: Arc<Mutex<WalletHandler>>,
        blocks: Arc<Mutex<HashMap<String, BlockMessage>>>,
        transactions: Arc<Mutex<HashMap<String, TXMessage>>>,
    ) {
        match message {
            InterfaceMessages::SendTransaction(send_transaction_node) => {
                Self::receive_send_transaction_order(
                    send_transaction_node,
                    sender_to_interface,
                    Arc::clone(&wallet_handler),
                    peers,
                );
            }
            InterfaceMessages::AddWalletOrder(add_wallet_node) => {
                Self::receive_add_wallet_order(
                    add_wallet_node,
                    sender_to_interface,
                    Arc::clone(&wallet_handler),
                );
            }
            InterfaceMessages::WalletSwitch(switch_node) => {
                Self::receive_switch_request(
                    switch_node,
                    sender_to_interface,
                    Arc::clone(&wallet_handler),
                );
            }
            InterfaceMessages::InclusionProof(block, transaction) => {
                Self::receive_inclusion_request(
                    block,
                    transaction,
                    sender_to_interface,
                    Arc::clone(&blocks),
                    Arc::clone(&transactions),
                );
            }
            InterfaceMessages::Close(_) => {
                Self::receive_save_order(Arc::clone(&wallet_handler));
            }
            _ => {}
        }
    }

    pub fn update_balance(&mut self, utxo: &UtxoCollector) {
        if let Ok(mut wallets) = self.wallet_handler.lock() {
            wallets.add_utxo_to_wallets(utxo);
        }
    }

    /// Save the information on disk
    fn save(wallet_handler: Arc<Mutex<WalletHandler>>) {
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("./storage/wallets.txt")
        {
            if let Ok(wallets) = wallet_handler.lock() {
                for wallet in wallets.wallets.iter() {
                    if file
                        .write_all(format!("{}\n", wallet.get_all_data()).as_bytes())
                        .is_ok()
                    {
                        println!("Error escribiendo en el archivo");
                    }
                }
            }
            if file.flush().is_ok() {}
        }
    }

    /// Returns an option containing a Wallet Handler if there is data in the wallets file and none if it was empty
    fn open() -> Option<WalletHandler> {
        let mut wallets = Vec::new();
        if let Ok(file) = File::open("./storage/wallets.txt") {
            if let Ok(metdata) = file.metadata() {
                if metdata.len() == 0 {
                    return None;
                }
            }
            let reader = BufReader::new(file);
            for line_result in reader.lines().flatten() {
                let parts: Vec<&str> = line_result.split(',').collect();
                if let Ok(id) = parts[0].parse() {
                    if let Ok(private) = parts[2].parse() {
                        if let Ok(balance) = parts[3].parse() {
                            wallets.push(Wallet::open(id, parts[1].to_string(), private, balance));
                        }
                    }
                }
            }
        }
        Some(WalletHandler::open(wallets))
    }

    /// Handle messages with interface --------------------------------

    /// Sends the command to open the interface with pre-existing data.
    fn open_interface(&self, open_signal: Sender<InterfaceMessages>) {
        if let Ok(wallets) = self.wallet_handler.lock() {
            let message = InterfaceMessages::Open(wallets.get_all_data());
            if open_signal.send(message).is_ok() {}
        }
    }

    /// Receives the command to save the wallets in file.
    fn receive_save_order(wallet_handler: Arc<Mutex<WalletHandler>>) {
        Self::save(wallet_handler);
        process::exit(1);
    }

    /// Receives the command to create a transaction.
    pub fn receive_send_transaction_order(
        transaction_order: (String, String, i32, i32),
        sender_to_interface: Sender<InterfaceMessages>,
        wallet_handler: Arc<Mutex<WalletHandler>>,
        peers: Vec<String>,
    ) {
        // Order of the tuple: (address, label, amount, fee)
        println!("recibe el pedido de transaccion");
        if let Ok(mut wallets) = wallet_handler.lock() {
            println!("lee las wallets de largo {:?}", wallets.wallets.len());

            if let Ok(transaction) = wallets.create_transaction(transaction_order) {
                println!("Transaction created: {:?}", transaction);
                Self::connect_streams(
                    peers,
                    transaction,
                    sender_to_interface,
                    wallets.actual_wallet_get_data(),
                );
            }
        }
    }

    /// Receives the command to add a wallet.
    fn receive_add_wallet_order(
        add_wallet_order: (String, String),
        sender_to_interface: Sender<InterfaceMessages>,
        wallet_handler: Arc<Mutex<WalletHandler>>,
    ) {
        if let Ok(mut wallets) = wallet_handler.lock() {
            if wallets.new_wallet(add_wallet_order.clone()) {
                let message = InterfaceMessages::WalletName(add_wallet_order.0);
                if sender_to_interface.send(message).is_ok() {}
            }
        }
    }

    /// Receives the command to switch wallets.
    fn receive_switch_request(
        switch_wallet_order: String,
        sender_to_interface: Sender<InterfaceMessages>,
        wallet_handler: Arc<Mutex<WalletHandler>>,
    ) {
        if let Ok(mut wallets) = wallet_handler.lock() {
            wallets.switch_wallet(switch_wallet_order);
            let message = InterfaceMessages::ActualWallet(wallets.actual_wallet_get_data());
            if sender_to_interface.send(message).is_ok() {}
        }
    }

    /// Receives a proof of inclusion request
    fn receive_inclusion_request(
        block: Vec<String>,
        transaction: Vec<String>,
        sender_to_interface: Sender<InterfaceMessages>,
        blocks: Arc<Mutex<HashMap<String, BlockMessage>>>,
        transactions: Arc<Mutex<HashMap<String, TXMessage>>>,
    ) {
        let mut include = false;
        if let Ok(list_blocks) = blocks.lock() {
            if let Some(b) = list_blocks.get(&block[0]) {
                if let Ok(list_txs) = transactions.lock() {
                    if let Some(tx) = list_txs.get(&transaction[3]) {
                        include = MerkleTree::proof_of_inclusion(b, tx);
                    }
                }
            }
        }
        let message = InterfaceMessages::InclusionProofResult(include);
        if sender_to_interface.send(message).is_ok() {}
    }

    fn connect_streams(
        mut peers: Vec<String>,
        transaction: TXMessage,
        sender_to_interface: Sender<InterfaceMessages>,
        wallet_data: HashMap<String, String>,
    ) {
        let mut can_sender_to_interface = false;
        peers.push("35.195.234.115:18333".to_string());
        peers.push("44.192.10.119:18333".to_string());
        peers.push("217.26.47.27:18333".to_string());
        peers.push("178.128.251.37:18333".to_string());
        peers.push("173.249.8.236:18333".to_string());
        for address in peers.iter() {
            if let Ok(stream) = TcpStream::connect(address) {
                let cloned_transaction = transaction.clone();
                let cloned_sender_to_interface = sender_to_interface.clone();
                writer(&stream);
                if cloned_transaction.send(&stream).is_ok() && !can_sender_to_interface {
                    println!("aca entra una vez {:?}", transaction);
                    let wallet_data = wallet_data.clone();
                    can_sender_to_interface = true;
                    let message = InterfaceMessages::MyTransactions(cloned_transaction);
                    if cloned_sender_to_interface.send(message).is_ok() {}
                    let message = InterfaceMessages::ActualWallet(wallet_data);
                    if sender_to_interface.send(message).is_ok() {}
                }
            }
        }
    }
}
