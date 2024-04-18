use glib::MainContext;

use super::connection_manager::peers_connection::{
    get_all_headers, get_blocks, get_headers, reader, save_blocks, writer,
};
use super::interface::interface_communicator::InterfaceMessages;
use crate::interface::interface_handler::InterfaceHandler;
use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::get_headers_message::GetHeadersMessage;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::inv::Inv;
use crate::message_structs::cmpct_block::CmpctBlock;
use crate::message_structs::merkel_block::MerkleBlock;
use crate::message_structs::outpoint::Outpoint;
use crate::message_structs::ping_or_pong::PingOrPong;
use crate::message_structs::block_txn::BlockTxn;
use crate::message_structs::tx_message::TXMessage;
use crate::node::connection_manager::peers_connection::{deserialize_message_from_client, handshake};

use crate::message_structs::get_block_txn::GetBlockTxn;
use crate::node::validation_engine::hashes::header_calculate_doublehash_array_be;
use crate::node::validation_engine::merkles::merkle_tree::MerkleTree;
use crate::utils::array_tools::u8_array_to_hex_string;
use crate::utils::commands::{get_type, MessageType};
use crate::utils::configs::config::get_server_seed;
use crate::utils::script_tools::bitcoin_address_in_b58_output;
use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpStream, TcpListener};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, MutexGuard};
use std::{thread, env};
//use crate::message_structs::filter_load_message::FilterLoadMessage;
use crate::message_structs::filter_load_message::FilterLoadMessage;
use crate::message_structs::inv_or_get_data_message::InvOrGetDataMessage;
use crate::message_structs::output::Output;
use crate::node::interface::interface_communicator::InterfaceCommunicator;
use crate::node::peer_discovery::obtain_peers::obtain_peers;
use crate::node::storage_engine::storage_manager::StorageManager;
use crate::node::utxo_collector::UtxoCollector;
use crate::utils::logger::Logger;
use crate::{
    message_structs::compact_size::*, message_structs::headers_message::*,
    utils::commands::match_command,
};
use glib::Sender as InterfaceSender;
use std::error::Error;
use std::sync::Mutex;
use std::thread::JoinHandle;
type Storage = Arc<Mutex<StorageManager>>;
type SenderLogClone = Arc<Mutex<Sender<(String, Vec<u8>)>>>;
type SenderInterface = InterfaceSender<InterfaceMessages>;
type StartInterfaceElements = (Vec<JoinHandle<()>>, SenderInterface);

pub const MAX_OUTBOUND_CONNECTIONS: usize = 30;

pub struct Handles {
    pub storage_blocks_handler: JoinHandle<()>,
    pub handles_interface: Vec<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct StorageMutex {
    pub storage_manager_sender: Arc<Mutex<StorageManager>>,
    pub sender_blocks: Arc<Mutex<Sender<BlockMessage>>>,
    pub storage_manager_merkles: Arc<Mutex<StorageManager>>
}

pub struct NodeComu {
    pub blocks_to_read: Arc<Mutex<Vec<InvOrGetDataMessage>>>,
    pub merkel_to_read: Arc<Mutex<Vec<InvOrGetDataMessage>>>,
    pub utxo_mutex: Arc<Mutex<HashMap<[u8; 32], Vec<Output>>>>,
    //pub last_header: Arc<Mutex<[u8; 32]>>,
    pub flag: Arc<Mutex<bool>>,
}

impl Clone for NodeComu {
    fn clone(&self) -> Self {
        NodeComu {
            blocks_to_read: self.blocks_to_read.clone(),
            merkel_to_read: self.merkel_to_read.clone(),
            utxo_mutex: self.utxo_mutex.clone(),
            //last_header: self.last_header.clone(),
            flag: self.flag.clone(),
        }
    }
}

impl NodeComu {
    pub fn lock_pass_block(&self) -> Option<MutexGuard<Vec<InvOrGetDataMessage>>> {
        let mut lock = self.blocks_to_read.lock();
        while lock.is_err() {
            lock = self.blocks_to_read.lock();
        }

        let get_data_vector = match lock {
            Ok(v) => v,
            Err(_) => {
                return None;
            }
        };

        Some(get_data_vector)
    }

    pub fn lock_pass_merkel(&self) -> MutexGuard<Vec<InvOrGetDataMessage>> {
        let get_data_merkel_vector = match self.merkel_to_read.lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error get_data_merkel_vector:{:?}", e);
            }
        };
        get_data_merkel_vector
    }

    pub fn lock_pass_utxo(&self) -> MutexGuard<HashMap<[u8; 32], Vec<Output>>> {
        let utxo_set = match self.utxo_mutex.lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error utxo_set:{:?}", e);
            }
        };
        utxo_set
    }
}

/// BitcoinNode is the main struct that holds all the information about the node.
/// It holds its blocks and its peers.
pub struct BitcoinNode {
    pub blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
    pub merkle_blocks: Arc<Mutex<HashMap<[u8;32],MerkleBlock>>>,
    pub headers: Arc<Mutex<Vec<Vec<u8>>>>,
    pub tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
    pub last_header:Arc<Mutex<[u8;32]>>,
    peers: Option<Vec<String>>,
    utxo_collector: UtxoCollector,
    interface_communicator: InterfaceCommunicator,
    total_blocks_to_receive: Arc<Mutex<(usize, bool)>>,
    is_client: bool, // This does not give the node any special behaviour, it's just to allow testing of the primary node
}

impl Default for BitcoinNode {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BitcoinNode {
    fn clone(&self) -> Self {
        BitcoinNode {
            peers: self.peers.clone(),
            blocks: self.blocks.clone(),
            merkle_blocks: self.merkle_blocks.clone(),
            headers: self.headers.clone(),
            tx:self.tx.clone(),
            last_header:self.last_header.clone(),
            utxo_collector: self.utxo_collector.clone(),
            interface_communicator: self.interface_communicator.clone(),
            total_blocks_to_receive: self.total_blocks_to_receive.clone(),
            is_client: self.is_client,
        }
    }
}

impl BitcoinNode {
    /// It creates a new BitcoinNode. No peers or blocks are initialized.
    pub fn new() -> BitcoinNode {
        // Initialization without peers
        BitcoinNode {
            peers: None,
            blocks: Arc::new(Mutex::new(HashMap::new())),
            merkle_blocks: Arc::new(Mutex::new(HashMap::new())),
            headers:Arc::new(Mutex::new(vec![])),
            tx:Arc::new(Mutex::new(HashMap::new())),
            last_header:Arc::new(Mutex::new([0;32])),
            utxo_collector: UtxoCollector::new(),
            interface_communicator: InterfaceCommunicator::new(),
            total_blocks_to_receive: Arc::new(Mutex::new((0, false))),
            is_client: false,
        }
    }

    /// It creates a new BitcoinNode with peers. No blocks are initialized.
    /// pasarle los mutex de bitnode a build connections
    pub fn build() -> Option<BitcoinNode> {
        let mut node = BitcoinNode::new();

        let peers = Self::build_connections(node.blocks.clone(), node.merkle_blocks.clone(), node.headers.clone(),node.tx.clone());
        node.peers = peers;
        node.is_client = Self::is_client();
        
        match node.start() {
            Ok(_) => Some(node),
            Err(_) => None,
        }
    }

    fn is_client() -> bool {
        let args: Vec<String> = env::args().collect();
        println!("node is client: {:?}", args.len() == 3);
        args.len() == 3
    }

    fn build_connections(blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
        merkle_blocks: Arc<Mutex<HashMap<[u8;32],MerkleBlock>>>,
        headers: Arc<Mutex<Vec<Vec<u8>>>>,
        tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,) -> Option<Vec<String>> {
        let server_seed = match get_server_seed() {
            Ok(seed) => seed,
            Err(_) => todo!(),
        };
        let args: Vec<String> = env::args().collect();
        println!("mis args son {:?}", args);
        let server_address = format!("{}:{}", server_seed, args[1]);
        Self::build_server(server_address,blocks,merkle_blocks, headers,tx);
        let client_address: Option<String>=if args.len() == 3 {
            Some(format!("{}:{}", server_seed, args[2]))
        } else {
            None
        };
        let peers = match obtain_peers(client_address) {
            Ok(nodes) => Some(nodes),
            Err(_) => {
                return None;
            }
        };
        peers
    }

    fn build_server(seed: String,
        blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
        merkle_blocks: Arc<Mutex<HashMap<[u8;32],MerkleBlock>>>,
        headers: Arc<Mutex<Vec<Vec<u8>>>>,
        tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,) {
        thread::spawn(move || {
            println!("se queda esperando conexion en {}", seed);
            if let Ok(listener) = TcpListener::bind(seed) {
                for stream in listener.incoming() {
                    let blocks = blocks.clone();
                    let headers = headers.clone();
                    let merkles = merkle_blocks.clone();
                    let tx =tx.clone();
                    println!("aca no entra nunca");
                    match stream {
                        Ok(stream) => {
                            thread::spawn(move || {
                                Self::handle_client(stream,blocks, merkles, headers, tx);
                            });
                        }
                        Err(err) => {
                            eprintln!("Error al aceptar la conexión del cliente: {}", err);
                        }
                    }
                }
            }
        });
    }

    fn handle_client(mut stream: TcpStream,
        blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
        merkle_blocks: Arc<Mutex<HashMap<[u8;32],MerkleBlock>>>,
        headers: Arc<Mutex<Vec<Vec<u8>>>>,
        tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,) {
        let mut buffer = vec![0u8; 24];
    
        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        println!("el cliente cerro la conexion");
                        break;
                    }
                    println!("el buffer que me llega es {:?}", buffer.to_vec());
                    //println!("el buffer es de tipo  {:?}", deserialize_message_from_client(&mut buffer.to_vec(), &stream));
                    let (message_type,mut vector) = deserialize_message_from_client(&mut buffer.to_vec(), &stream);
                    match message_type {
                        MessageType::VersionMessage => {
                            println!("Sending version to client");
                            handshake(&stream);
                        }
                        MessageType::Mempool => {
                            Self::mempool(&mut stream,&mut vector,tx.clone());
                        }
                        MessageType::GetHeadersMessage => {
                            Self::get_headers(
                                &mut stream,
                                &mut vector,
                                headers.clone()
                            )
                        }
                        MessageType::GetDataMessage => Self::get_data_message(
                            &mut stream,
                            &mut vector,
                            tx.clone(),
                            blocks.clone(),
                            merkle_blocks.clone()  
                        ),
                        MessageType::GetBlockTxn=>Self::get_block_tx(&mut stream,&mut vector,blocks.clone()),

                        _ => {}
                    }
                    //println!("el mensaje es de tipo {:?}", get_type(buffer));

                    //Procesar al cliente (recibir enviar datos)
                }
                Err(err) => {
                    eprintln!("Error al leer los datos del cliente: {}", err);
                    break;
                }
            }
        }
    }

    /// It starts the node. It connects to the peers and starts downloading the blocks.
    /// It returns an error if it cannot connect to the peers.
    /// It starts the node's behaviour and triggers the interface.
    pub(self) fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let storage_manager_headers = StorageManager::new_header_storage(self.is_client);
        let storage_manager_blocks = StorageManager::new_blocks_storage(self.is_client);
        let storage_manager_merkles = StorageManager::new_merkle_storage(self.is_client);

        let peers: Vec<String> = match &self.peers {
            Some(p) => p.to_vec(),
            None => {println!("error con peers");
                return Err("No peers".into());
            }
        };

        let headers_available = match storage_manager_headers.file_is_empty() {
            Ok(v) => !v,
            Err(_e) => {println!("error chequeando el archivo");
                return Err("Error checking if storage manager has headers".into());
            }
        };
        //UTXO set
        let mut utxo_set: HashMap<[u8; 32], Vec<Output>> = HashMap::new();
        let last_block = self.blocks_available(storage_manager_blocks, &mut utxo_set);
        self.merkleblocks_available(&storage_manager_merkles);

        let (get_data_block, get_data_merkel, last_header) =
            self.headers_store(headers_available, &storage_manager_headers, last_block);

        self.variable_creation(
            (storage_manager_headers,
            storage_manager_merkles),
            get_data_block,
            get_data_merkel,
            utxo_set,
            peers,
            last_header,
        );

        Ok(())
    }

    ///Check if there are blocks store
    /// if there are it saves the last and updates the utxo_set
    /// if there is none i return a empty block
    fn blocks_available(
        &mut self,
        storage_manager_blocks: StorageManager,
        utxo_set: &mut HashMap<[u8; 32], Vec<Output>>,
    ) -> BlockMessage {
        let mut block = Self::get_empty_block();

        
        let blocks_available = match storage_manager_blocks.file_is_empty() {
            Ok(v) => !v,
            Err(_e) => {
                println!("Error checking if storage manager has blocks");
                return block;
            }
        };

        if blocks_available {
            let merkles_strings = match storage_manager_blocks.read_csv_file() {
                Ok(v) => v,
                Err(_v) => vec![vec![]],
            };
            let mut blocks = match self.blocks.lock(){
                Ok(v)=>v,
                Err(_v)=>return block,
            };
            //println!("lectura {:?}", merkles_strings);
            for i in merkles_strings {
                println!("leyendo bloque");
                let mut used_tx: HashMap<[u8; 32], Vec<u32>> = HashMap::new();
                block = BlockMessage::blocks_from_str(i);
                blocks.insert(header_calculate_doublehash_array_be(&block.get_block_header()).unwrap_or([0; 32]), block.serialize());
                let mut txid = block.get_ids();
                Self::add_to_utxo_available(utxo_set, &mut block, &mut txid);
                Self::add_to_used_tx(&mut used_tx, &mut block);
                Self::remove_use_tx(utxo_set, &mut used_tx);
            }
        };
        println!("last_block{:?}", block);
        block
    }

    /// Check if there are merkle blocks stored
    /// if there are it updates the merkle_block hashmap
    fn merkleblocks_available(
        &mut self,
        storage_manager_merkleblocks: &StorageManager,
    ) {
        
        let merkleblocks_available = match storage_manager_merkleblocks.file_is_empty() {
            Ok(v) => !v,
            Err(_e) => {
                println!("Error checking if storage manager has merkleblocks_available");
                return;
            }
        };

        if merkleblocks_available {
            println!("merkleblocks_available");
            let merkles_strings = match storage_manager_merkleblocks.read_csv_file() {
                Ok(v) => v,
                Err(_v) => vec![vec![]],
            };
            let mut merkle_blocks = match self.merkle_blocks.lock(){
                Ok(v)=>v,
                Err(_v)=>return,
            };

            for i in merkles_strings {
                println!("leyendo bloque");
                let merkle_block = match MerkleBlock::from_string_to_vec(i) {
                    Ok(v) => v,
                    Err(_v) => {
                        println!("Error reading merkleblock");
                        return;
                    }
                };

                merkle_blocks.insert(header_calculate_doublehash_array_be(&merkle_block.block_header).unwrap_or([0; 32]), merkle_block);
            }
        };
    }

    pub fn get_empty_block() -> BlockMessage {
        BlockMessage {
            block_header: BlockHeader {
                version: 0,
                previous_block_header_hash: [0; 32],
                merkle_root_hash: [0; 32],
                time: 0,
                n_bits: 0,
                nonce: 0,
            },
            tx_count: CompactSize {
                prefix: 0,
                number_vec: [0].to_vec(),
                number: 0,
            },
            transaction_history: vec![TXMessage::new(
                0,
                CompactSize {
                    prefix: 0,
                    number_vec: [0].to_vec(),
                    number: 0,
                },
                vec![],
                CompactSize {
                    prefix: 0,
                    number_vec: [0].to_vec(),
                    number: 0,
                },
                vec![],
                0,
            )],
        }
    }

    ///Checks if there are headers stored.
    /// if there are it creates the get_data for the missing block and saves the last header
    /// if there is not return two empty vectors and an array [0;32]
    /// if there are more blocks than headers last header will start from the last block
    fn headers_store(
        &mut self,
        headers_available: bool,
        storage_manager_headers: &StorageManager,
        last_block: BlockMessage,
    ) -> (Vec<InvOrGetDataMessage>, Vec<InvOrGetDataMessage>, [u8; 32]) {
        let mut headers;
        let mut get_data_block: Vec<InvOrGetDataMessage> = vec![];
        let mut get_data_merkel: Vec<InvOrGetDataMessage> = vec![];
        let mut last_header = [0; 32];
        if headers_available {
            let merkles_strings = match storage_manager_headers.read_csv_file() {
                Ok(v) => v,
                Err(_v) => vec![vec!["0".to_string()]],
            };
            //println!("lectura {:?}", merkles_strings);
            headers = match HeadersMessage::headers_from_str(merkles_strings) {
                Ok(v) => v,
                Err(_v) => HeadersMessage::new(
                    CompactSize::from_usize_to_compact_size(0),
                    vec![BlockHeader::new(0, [0u8; 32], [0u8; 32], 0, 0, 0)],
                ),
            };
            let mut headers_lock =match self.headers.lock(){
                Ok(v)=>v,
                Err(_v)=>return (get_data_block,get_data_merkel,last_header),
            };
            match headers.to_serialized(){
                Some(v)=>headers_lock.extend_from_slice(&v),
                None=>return(get_data_block,get_data_merkel,last_header),
            }
            let block_header = last_block.get_block_header();
            headers = headers.blocks_missing(block_header.clone());
            //println!("headers {:?}",headers);
            last_header = headers.last_hash();
            let last_header_block =
                header_calculate_doublehash_array_be(&block_header).unwrap_or([0; 32]);
            if last_header != last_header_block {
                get_data_block = headers.get_data_with_type(2);
                get_data_merkel = headers.get_data_with_type(3);
            }
            println!("headers Stored:{:?}",self.headers);
            println!("last_header{:?}", last_header);
        };
        (get_data_block, get_data_merkel, last_header)
    }

    ///Creates the variables that will be shared among the threads
    fn variable_creation(
        &mut self,
        storage_managers: (StorageManager, StorageManager),
        get_data_block: Vec<InvOrGetDataMessage>,
        get_data_merkel: Vec<InvOrGetDataMessage>,
        utxo_set: HashMap<[u8; 32], Vec<Output>>,
        peers: Vec<String>,
        last_header: [u8; 32],
    ) {
        //Interfaz y wallet
        let (handles_interface, sender_to_interface) = self.start_interface(peers.clone());

        //Store Headers
        let storage_manager_sender = Arc::new(Mutex::new(storage_managers.0));
        let storage_manager_merkles = Arc::new(Mutex::new(storage_managers.1));
        //Store Blocks
        let (sender, receiver) = std::sync::mpsc::channel::<BlockMessage>();
        let receiver_blocks: Arc<Mutex<Receiver<BlockMessage>>> = Arc::new(Mutex::new(receiver));
        let sender_blocks: Arc<Mutex<Sender<BlockMessage>>> = Arc::new(Mutex::new(sender));
        let storage_blocks_handler: JoinHandle<()> = save_blocks(
            receiver_blocks,
            sender_to_interface.clone(),
            Arc::clone(&self.interface_communicator.blocks),
            Arc::clone(&self.total_blocks_to_receive),
            self.is_client
        );

        //node comunication
        let blocks_to_read: Arc<Mutex<Vec<InvOrGetDataMessage>>> =
            Arc::new(Mutex::new(get_data_block));
        let merkel_to_read: Arc<Mutex<Vec<InvOrGetDataMessage>>> =
            Arc::new(Mutex::new(get_data_merkel));
        let utxo_mutex: Arc<Mutex<HashMap<[u8; 32], Vec<Output>>>> = Arc::new(Mutex::new(utxo_set));
        let last_header: Arc<Mutex<[u8; 32]>> = Arc::new(Mutex::new(last_header));
        self.last_header = last_header;

        let handels = Handles {
            storage_blocks_handler,
            handles_interface,
        };

        let storage: StorageMutex = StorageMutex {
            storage_manager_sender,
            sender_blocks,
            storage_manager_merkles
        };
        let node_coms: NodeComu = NodeComu {
            blocks_to_read,
            merkel_to_read,
            utxo_mutex,
            flag: Arc::new(Mutex::new(false)),
        };

        self.addresses_connection(peers, sender_to_interface, storage, node_coms, handels);
    }

    ///Clones all the data previously created for threads
    fn addresses_connection(
        &mut self,
        mut peers: Vec<String>,
        sender_to_interface: SenderInterface,
        storage: StorageMutex,
        node_coms: NodeComu,
        handles: Handles,
    ) {
        // Logger
        let logger: Logger = Logger::default();
        let logger_handler: JoinHandle<()> = logger.start();

        // Extending known working nodes
        peers.extend([
            "35.195.234.115:18333".to_string(),
            "44.192.10.119:18333".to_string(),
            "217.26.47.27:18333".to_string(),
            "178.128.251.37:18333".to_string(),
            "173.249.8.236:18333".to_string(),
        ]);
        let flag = &node_coms.flag;

        for address in peers.iter().cycle() {
            println!("address iterated: {:?}", address);
            match flag.lock() {
                Ok(mut flag) => {
                    *flag = false;
                }
                Err(e) => {
                    println!("Error locking flag: {:?}", e);
                    return;
                }
            }
            let sender_log_clone = logger.get_sender_clone();
            let flag_clone = flag.clone();
            let sender_to_interface = sender_to_interface.clone();
            let storage = storage.clone();
            let node_coms = node_coms.clone();

            self.node_connection(
                address.to_string(),
                flag_clone,
                sender_log_clone as SenderLogClone,
                sender_to_interface,
                storage,
                node_coms,
            );
            println!("Function node_connection finished");
        }

        Self::join_handles(
            handles.storage_blocks_handler,
            logger_handler,
            handles.handles_interface,
        );
    }

    ///Once the connection has finished for every node it joins the handles used
    fn join_handles(
        storage_blocks_handler: JoinHandle<()>,
        logger_handler: JoinHandle<()>,
        handles_interface: Vec<JoinHandle<()>>,
    ) {
        match storage_blocks_handler.join() {
            Ok(_) => println!("StoreBlocks.join: Thread finished successfully"),
            Err(e) => println!("Thread panicked: {:?}", e),
        }

        match logger_handler.join() {
            Ok(_) => println!("Handle.join: Thread finished successfully"),
            Err(e) => println!("Thread panicked: {:?}", e),
        }

        for handle in handles_interface {
            match handle.join() {
                Ok(_) => println!("Interface.join: Thread finished successfully"),
                Err(e) => println!("Thread panicked: {:?}", e),
            }
        }
    }

    ///Starts the connection with the node
    fn node_connection(
        &mut self,
        address: String,
        flag_clone: Arc<Mutex<bool>>,
        sender_log_clone: SenderLogClone,
        sender_to_interface: SenderInterface,
        storage: StorageMutex,
        node_coms: NodeComu,
    ) {
        println!("Function: node_connection for address");
        let stream = match TcpStream::connect(address) {
            Ok(s) => s,
            Err(_) => return,
        };

        // create write stream
        let mut write_stream = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };

        writer(&write_stream);

        get_headers(&mut write_stream, &self.last_header);

        self.node_internal_threads(
            stream,
            sender_log_clone,
            flag_clone,
            sender_to_interface,
            storage,
            node_coms,
        );
        println!("Function: node_connection_threads for address finished");
    }

    ///Handles the threads in which our connection is based
    fn node_internal_threads(
        &mut self,
        stream: TcpStream,
        sender_log_clone: SenderLogClone,
        flag: Arc<Mutex<bool>>,
        sender_to_interface: SenderInterface,
        storage: StorageMutex,
        node_coms: NodeComu,
    ) {
        println!("Function: node_internal_threads");
        let mut handles = vec![];
        let (tx_read, rx_read) = channel::<Vec<u8>>();
        //to aviod the readexact issue
        let (sender, receiver) = channel::<u8>();

        let read_stream = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };

        let cloned_flag_read = Arc::clone(&flag);

        let read_handle = thread::spawn(move || {
            reader(
                cloned_flag_read,
                read_stream,
                tx_read,
                sender_log_clone,
                receiver,
            );
            println!("reader thread finished");
        });
        handles.push(read_handle);

        let write_block_stream = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => return,
        };

        let node = Arc::new(Mutex::new(self.clone()));
        let write_handle = thread::spawn(move || {
            Self::unlocking_node(
                Arc::clone(&node),
                rx_read,
                write_block_stream,
                sender_to_interface,
                storage,
                node_coms,
                sender,
            );
            println!("writer thread finished");
            match flag.lock() {
                Ok(mut flag) => {
                    *flag = true;
                }
                Err(e) => {
                    println!("Error locking flag: {:?}", e);
                }
            }
        });
        handles.push(write_handle);

        println!("waiting on handles: {:?}", handles);
        for handle in handles {
            match handle.join() {
                Ok(_) => println!("Threadpool.join: Thread finished successfully"),
                Err(e) => println!("Thread panicked: {:?}", e),
            }
        }
    }

    fn unlocking_node(
        node: Arc<Mutex<BitcoinNode>>,
        rx_read: Receiver<Vec<u8>>,
        write_block_stream: TcpStream,
        sender_to_interface: SenderInterface,
        storage: StorageMutex,
        node_coms: NodeComu,
        sender: Sender<u8>,
    ) {
        if let Ok(mut node_lock) = node.lock() {
            node_lock.messages_handler(
                rx_read,
                write_block_stream,
                sender_to_interface,
                storage,
                node_coms,
                sender,
            );
            println!("unlocking_node done")
        }
    }

    /// handles the arrival of messages and what messages should be send
    fn messages_handler(
        &mut self,
        rx_read: Receiver<Vec<u8>>,
        mut write_block_stream: TcpStream,
        sender_to_interface: SenderInterface,
        storage: StorageMutex,
        node_coms: NodeComu,
        sender: Sender<u8>,
    ) {
        let mut get_data_vector = match node_coms.lock_pass_block() {
            Some(g) => g,
            None => return,
        };
        let mut get_data_merkel_vector = node_coms.lock_pass_merkel();
        let mut utxo_set = node_coms.lock_pass_utxo();
        println!("Function: messages_handler_locks_pass");
        let stop_flag = &node_coms.flag;
        let mut _get_data = InvOrGetDataMessage::new(
            CompactSize::from_usize_to_compact_size(0),
            vec![Inv::new(0, [0; 32])],
        );
        let mut _get_data_merkel = InvOrGetDataMessage::new(
            CompactSize::from_usize_to_compact_size(0),
            vec![Inv::new(0, [0; 32])],
        );
        let mut data_loaded = false;
        loop {
            let mut lock_stop_flag = stop_flag.lock();
            while lock_stop_flag.is_err() {
                lock_stop_flag = stop_flag.lock();
            }
            match lock_stop_flag {
                Ok(stop_flag) => {
                    if *stop_flag {
                        break;
                    }
                }
                Err(e) => {
                    println!("Error locking stop_flag: {:?}", e);
                }
            }
            let Ok(_result) = sender.send(1) else {
                return;
            };

            let mut reading_headers = false;
            let mut tx_recieved = false;
            let mut vector = match rx_read.recv() {
                Ok(v) => v,
                Err(v) => {
                    println!("Error receiving message with recv, {:?}", v);
                    break;
                }
            };
            let command = match vector.drain(0..12).collect::<Vec<u8>>().try_into() {
                Ok(a) => a,
                Err(_) => panic!("Failed to convert vector to array"),
            };
            match get_type(&command) {
                MessageType::End => {
                    let Ok(_result) = sender.send(0) else {return};
                    return;
                }
                MessageType::HeadersMessage => {
                    reading_headers = self.headers_message(
                        &mut write_block_stream,
                        vector,
                        sender_to_interface.clone(),
                        &storage.storage_manager_sender,
                        &mut get_data_vector,
                        &mut get_data_merkel_vector,
                        //&node_coms.last_header,
                    )
                }
                MessageType::BlockMessage => {
                    data_loaded = self.block_message(
                        &mut vector,
                        &storage.sender_blocks,
                        &mut utxo_set,
                        sender_to_interface.clone(),
                        &mut get_data_vector,
                        &mut get_data_merkel_vector,
                    )
                }
                MessageType::MerkleBlock => {
                    self.merkel_block(&mut vector, storage.storage_manager_merkles.clone(), &mut get_data_vector, &write_block_stream)
                }
                MessageType::InvMessage => Self::inv_message(
                    &mut write_block_stream,
                    &mut vector,
                    &data_loaded,
                    &reading_headers,
                ),
                MessageType::Tx => {
                    self.tx_message_was_received(&mut vector, sender_to_interface.clone());
                    tx_recieved = true;
                }
                MessageType::Ping => {
                    let ping = PingOrPong::deserialize(&mut vector);
                    let _result = ping.send_pong(&write_block_stream);
                }
                MessageType::NotFound => {
                    _get_data = InvOrGetDataMessage::new(
                        CompactSize::from_usize_to_compact_size(0),
                        vec![Inv::new(0, [0; 32])],
                    );
                    _get_data_merkel = InvOrGetDataMessage::new(
                        CompactSize::from_usize_to_compact_size(0),
                        vec![Inv::new(0, [0; 32])],
                    );
                }
                _ => Self::other_message(&command, &mut vector),
            };
            println!(
                "get_data_vector.is_empty():{}",
                get_data_merkel_vector.len()
            );
            if (!data_loaded) && !(get_data_merkel_vector.is_empty()) && (!reading_headers) {
                self.block_work(
                    &mut get_data_merkel_vector,
                    get_data_vector.len(),
                    &write_block_stream,
                );
                data_loaded = true;
            }

            println!("get_data_vector.is_empty():{}", get_data_vector.len());
            if get_data_vector.is_empty() && (!reading_headers) {
                println!("update");
                println!("HASH UPDATE");
                self.create_address_utxo(&mut utxo_set);
                if !tx_recieved {
                    let mempool = BitcoinMessageHeader::mempool();
                    let _result = mempool.send(&write_block_stream);
                    println!("\n enviado {:?} \n", mempool);
                }
            }
        }
    }

    ///A header message is recieved and depending on its len it decides what to do
    fn headers_message(
        &mut self,
        read_stream: &mut TcpStream,
        message: Vec<u8>,
        sender_to_interface: SenderInterface,
        storage_manager: &Storage,
        blocks_to_read: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
        merkel_to_read: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
        //last_header: &Arc<Mutex<[u8; 32]>>,
    ) -> bool {
        println!("Function: headers_message");
        let mut response = false;
        if message.len() > 10 {
            let header = match get_all_headers(
                read_stream,
                message,
                sender_to_interface,
                storage_manager,
                blocks_to_read,
                merkel_to_read,
                &self.last_header,
            ){
                Some(v) =>v,
                None=> return true,
            };
            let mut headers =match self.headers.lock(){
                Ok(v)=>v,
                Err(_v)=>return response,
            };
            match header.to_serialized(){
                Some(v)=>headers.extend_from_slice(&v),
                None=>return true,
            }


            if header.get_headers().len() < 2000 {
                if let Ok(mut total_blocks) = self.total_blocks_to_receive.lock() {
                    if !total_blocks.1 {
                        println!("aca entra y envia {:?}", blocks_to_read.len());
                        total_blocks.0 = blocks_to_read.len();
                    }
                }
            }
            
            response = true
        } else {
            get_headers(read_stream, &self.last_header);
        }
        response
    }

    ///A block message is recieved, send to storage and save for future use
    fn block_message(
        &mut self,
        vector: &mut Vec<u8>,
        sender_blocks_clone: &Arc<Mutex<Sender<BlockMessage>>>,
        utxo_set: &mut MutexGuard<HashMap<[u8; 32], Vec<Output>>>,
        sender_to_interface: SenderInterface,
        get_data_vector: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
        get_data_merkel_vector: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
    ) -> bool {
        println!("Function: block_message");
        if !get_data_vector.is_empty() {
            get_data_vector.remove(0);
        }
        if !get_data_merkel_vector.is_empty() {
            get_data_merkel_vector.remove(0);
        }

        let mut used_tx: HashMap<[u8; 32], Vec<u32>> = HashMap::new();
        let mut vector_copy = vector.clone();
        let sender_blocks_clone = sender_blocks_clone.clone();

        get_blocks(sender_blocks_clone, vector);
        let mut block = match BlockMessage::deserialize(&mut vector_copy) {
            Ok(v) => v,
            Err(_) => BlockMessage::new(
                BlockHeader {
                    version: 0,
                    previous_block_header_hash: [0; 32],
                    merkle_root_hash: [0; 32],
                    time: 0,
                    n_bits: 0,
                    nonce: 0,
                },
                CompactSize::from_usize_to_compact_size(0),
                vec![],
            ),
        };
        let mut txid = block.get_ids();

        self.add_to_utxo(utxo_set, &mut block, &mut txid, sender_to_interface);
        Self::add_to_used_tx(&mut used_tx, &mut block);
        Self::remove_use_tx(utxo_set, &mut used_tx);
        false
    }

    fn merkel_block(
        &mut self,
        vector: &mut Vec<u8>,
        storage: Arc<Mutex<StorageManager>>,
        get_data_vector: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
        write_block_stream: &TcpStream,
    ) {
        println!("Function: merkel_block");

        let merkle_msg = match MerkleBlock::deserialize(vector) {
            Ok(v) => v,
            Err(_) => MerkleBlock::new(
                BlockHeader {
                    version: 0,
                    previous_block_header_hash: [0; 32],
                    merkle_root_hash: [0; 32],
                    time: 0,
                    n_bits: 0,
                    nonce: 0,
                },
                0,
                CompactSize::from_usize_to_compact_size(0),
                vec![],
                CompactSize::from_usize_to_compact_size(0),
                vec![],
            ),
        };

        println!("\nmerklefix Merkle tree received: {:?}\n", merkle_msg);

        if merkle_msg.hashes.is_empty() {
            return;
        }

        if MerkleTree::merkle_block_is_valid(&merkle_msg) {
            println!(
                "Merkle tree is valid with transaction_count: {:?}",
                merkle_msg.transaction_count
            );

            let block_header = merkle_msg.clone().block_header;
            let mut  merkle_blocks = match self.merkle_blocks.lock(){
                Ok(v)=>v,
                Err(_v)=>return,
            };
            merkle_blocks.insert(header_calculate_doublehash_array_be(&block_header).unwrap_or([0; 32]), merkle_msg.clone());

            // Save Merkle Block
            let mut storage = match storage.lock(){
                Ok(v)=>v,
                Err(_v)=>return,
            };
            _ = storage.save_merkle_data_to_file(merkle_msg);

            if let Some(origin) = get_data_vector.first() {
                let _result = origin.send_get_data(write_block_stream);
            }
            //println!("\n enviado {:?}\n", get_data);
        } else {
            println!(
                "Merkle tree is not valid with transaction_count: {:?}",
                merkle_msg.transaction_count
            );
        }
    }

    ///An inv_message is recieved and depending on its contents it gets a response or not
    fn inv_message(
        read_stream: &mut TcpStream,
        vector: &mut Vec<u8>,
        data_loaded: &bool,
        reading_headers: &bool,
    ) {
        let inv_message = match InvOrGetDataMessage::deserialize(vector) {
            Ok(v) => v,
            Err(_v) => return,
        };
        if !*data_loaded && !*reading_headers {
            inv_message.ask_for_tx(read_stream);
        }
    }
    
    fn get_data_message(
        read_stream: &mut TcpStream,
        vector: &mut Vec<u8>,
        tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
        blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,
        merkle_blocks: Arc<Mutex<HashMap<[u8;32],MerkleBlock>>>,

    ) {
        let get_data = match InvOrGetDataMessage::deserialize(vector) {
            Ok(v) => v,
            Err(_v) => return,
        };
        println!("getData recibido: {:?}",get_data);
        let txs = match tx.lock(){
            Ok(v)=>v,
            Err(_v)=>return
        };
        let block_lock = match blocks.lock(){
            Ok(v)=>v,
            Err(_v)=>return
        };
        let tx  = txs.clone();
        let blocks = block_lock.clone();
        for i in get_data.inv(){
           match i.inv_type(){
            1 => {
                if let Some(tx_to_send) = Self::_find_tx(tx.clone(),i.hash()) {
                    let _result=tx_to_send.send(read_stream);
                    println!("TX getdata sent {:?}",tx_to_send)
                }
                else {
                    let not_found = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(1), vec![i]);
                    let _result = not_found.send_not_found(read_stream);
                }
            },
            2 => {
                if let Some(block_to_send) = Self::_find_block(blocks.clone(),i.hash()) {
                    let _result=block_to_send.send(read_stream);
                    println!("block getdata sent {:?}",block_to_send)
                }
                else{
                    let not_found = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(1), vec![i]);
                    let _result = not_found.send_not_found(read_stream);
                }
            },
            3 => {

                // Buscarlo en el hash de Merkles y devolverlo

                let merkle_block_lock = match merkle_blocks.lock(){
                    Ok(v)=>v,
                    Err(_v)=>return
                };

                if let Some(merkle_block) = Self::_find_merkle(merkle_block_lock.clone(), i.hash()) {
                    let _result = merkle_block.send(read_stream);
                    println!("block getdata sent {:?}",merkle_block);
                }
                else if let Some(tx) = Self::_find_tx(tx.clone(), i.hash()) {
                    let _result = tx.send(read_stream);
                }
                else{
                    let not_found = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(1), vec![i]);
                    let _result = not_found.send_not_found(read_stream);
                }
            }
            4 => {
                if let Some(block_to_send) = Self::_find_block(blocks.clone(),i.hash()) {
                            let cmpt_block = CmpctBlock::create_cmpt_block(block_to_send,tx.clone());
                            let _result=cmpt_block.send(read_stream);
                            println!("cmpt block getdata sent {:?}",cmpt_block)
                }
                else{
                    let not_found = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(1), vec![i]);
                    let _result = not_found.send_not_found(read_stream);
                } 
            }
            _ => {},
            } 
        }
        
        // if !*data_loaded && !*reading_headers {
        //     inv_message.ask_for_tx(read_stream);
        // }
    }

    fn _find_block(mut blocks: HashMap<[u8; 32], Vec<u8>>, hash:  [u8; 32]) ->Option<BlockMessage> {
        if let Some(block_send) = blocks.get_mut(&hash){
            if let Ok(block_to_send) = BlockMessage::deserialize(block_send) {
                return Some(block_to_send);
            };
        };
        None
    }

    fn _find_merkle(mut merkles: HashMap<[u8; 32], MerkleBlock>, hash:  [u8; 32]) ->Option<MerkleBlock> {
        if let Some(block_send) = merkles.get_mut(&hash){
            return Some(block_send.clone());
        };
        None
    }

    fn _find_tx(mut txs: HashMap<[u8; 32], Vec<u8>>, hash:  [u8; 32]) ->Option<TXMessage> {
        if let Some(tx_send) = txs.get_mut(&hash){
            if let Ok(tx_to_send) = TXMessage::deserialize(tx_send) {
                return Some(tx_to_send);
            };
        };
        None
    }

    //if fail try reverse the hash
    fn get_block_tx(read_stream: &mut TcpStream,message:&mut Vec<u8>,blocks: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>){
        let get_block_tx = match GetBlockTxn::deserialize(message){ 
            Ok(v) => v,
            Err(_v) => return,
        };
        println!("getBlocksTX recibido: {:?}",get_block_tx);
        let block_lock = match blocks.lock(){
            Ok(v)=>v,
            Err(_v)=>return
        };
        let mut blocks = block_lock.clone();
        if let Some(block_serialized) = blocks.get_mut(&get_block_tx.block_hash()){
            let block = match BlockMessage::deserialize(block_serialized){
                Ok(v)=>v,
                Err(_v)=>return,
            };
            let mut txs_needed = vec![];
            let txs = block.get_tx();
            let mut indexes = get_block_tx.indexes();
            while !indexes.is_empty(){
                let index = indexes.remove(0).get_number() - 1;
                let tx =match txs.get(index){
                    Some(v)=>v.clone(),
                    None=>return,
                };
                txs_needed.push(tx);
            }
            let block_tx = BlockTxn{block_hash:get_block_tx.block_hash(),transactions_length:CompactSize::from_usize_to_compact_size(txs_needed.len()),transactions:txs_needed};
            let _result = block_tx.send(read_stream);
            println!("enviado {:?}",block_tx);
        };
        

    }

    //FIXME no lo logro hacer andar con todos los headeres y bloques descargados
    fn get_headers(
        read_stream: &mut TcpStream,
        message:&mut Vec<u8>,
        headers: Arc<Mutex<Vec<Vec<u8>>>>){
            let get_headrs = match GetHeadersMessage::deserialize(message){
                Ok(v)=>v,
                Err(_)=>return
            };
            println!("getHEaders recibido: {:?}",get_headrs);
            // habra que diferenciar segun nodo y servidor
            let headers_lock =  match headers.lock(){
                Ok(v)=>v,
                Err(_v)=>return,
            };
            let mut headers =headers_lock.clone();
            if !headers.is_empty(){
                let mut headers_hashes = vec![];
                for mut i in headers.clone(){
                    let block_header = match BlockHeader::deserialize(&mut i){
                        Ok(v)=>v,
                        Err(_v)=>return,
                    };
                    let block_hash  = match header_calculate_doublehash_array_be(&block_header){
                        Some(v)=>v,
                        None=>return,
                    };
                    //block_hash.reverse();
                    headers_hashes.push(block_hash);
                }
                let mut hashes = get_headrs.hashes();
                let mut not_sent =true;
                while not_sent && !hashes.is_empty(){
                    let hash = hashes.remove(0);
                    println!("hash actual:{:?}",hash);
                    println!("contains: {:?}",headers_hashes.contains(&hash));
                    if headers_hashes.contains(&hash)|| hash== [0;32]{
                        while not_sent && !headers_hashes.is_empty(){
                            let mut header_hash = [0;32];
                            if hash != [0;32]{
                                println!("headers left:{:?}",headers.len());
                                header_hash = headers_hashes.remove(0);
                                let _i = headers.remove(0);
                            };
                            if header_hash == hash{
                                let count;
                            if headers.len()>2000{
                                count=CompactSize::from_usize_to_compact_size(2000);
                                headers.resize(2000,vec![]);
                            }
                            else{
                                count =CompactSize::from_usize_to_compact_size( headers.len());
                            };
                            let mut message = vec![];
                            message.extend_from_slice(&count.serialize());
                            for i in headers.clone(){
                                message.extend_from_slice(&i);
                                message.push(0);
                            }
                            let headers = match HeadersMessage::deserialize(&mut message){
                                Ok(v)=>v,
                                Err(_e) => {  println!("error con los headers");
                                                            return},
                            };
                            if headers.count() > 0{
                                println!("headers A ENVIAR:{:?}",headers);
                                let _result =  headers.send(read_stream);
                            }
                            else{
                                let header_empty = BitcoinMessageHeader::empty_headers();
                                let _result = header_empty.send(read_stream);
                            }
                            
                            not_sent = false;
                        } 
                    }
                }
                else{
                    println!("no se encontro header")   
                }
                
            }
            }
            

        }

    fn mempool(read_stream: &mut TcpStream,
        _vector: &mut [u8],
        tx: Arc<Mutex<HashMap<[u8;32],Vec<u8>>>>,){
        println!("Mempool llego");
        let txs = match tx.lock(){
            Ok(v)=>v,
            Err(_v)=>return,
        };
        let values = txs.keys();
        let mut  vector = vec![];
        for i in values.clone(){
            vector.push(Inv::new(1, *i));
        }
        let inv = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(vector.len()),vector);
        let _result = inv.send_inv(read_stream);
    }

    /// A tx is recieved but is not confirmed yet so it checks if any wallet has an address involved in the tx
    fn tx_message_was_received(
        &mut self,
        vector: &mut Vec<u8>,
        sender_to_interface: SenderInterface,
    ) {
        let tx = match TXMessage::deserialize(vector) {
            Ok(v) => v,
            Err(_v) => return,
        };
        let mut txs_lock = match self.tx.lock(){
            Ok(v)=>v,
            Err(_v)=>return
        };
        txs_lock.insert(tx.get_id(), tx.serialize());

        if let Ok(mut txs) = self.interface_communicator.transactions.lock() {
            txs.insert(u8_array_to_hex_string(&tx.get_id()), tx.clone());
        }
        let belongs = self.is_user_tx(tx.clone());
        let message = InterfaceMessages::AllTransactions(false, tx, belongs);
        if sender_to_interface.send(message).is_ok() {}

        //Pertenece al usuario
    }

    fn is_user_tx(&self, tx: TXMessage) -> bool {
        let wallet = self.interface_communicator.wallet_handler.lock();
        let mut belongs = false;
        if wallet.is_ok() {
            let wallet_pass = match wallet.ok() {
                Some(v) => v,
                None => return false,
            };

            let wallet_pass = match wallet_pass.get_actual_wallet() {
                Some(v) => v,
                None => return belongs,
            };
            let address = wallet_pass.get_address();
            let output = tx.get_output();
            let input = tx.get_input();

            for i in output {
                let output_address = bitcoin_address_in_b58_output(&i.get_script());
                if address == output_address {
                    belongs = true;
                }
            }
            for j in input {
                let output_address = bitcoin_address_in_b58_output(&j.get_script());
                if address == output_address {
                    belongs = true;
                }
            }
        }
        belongs
    }

    ///Shows the user other messages that have arrived
    fn other_message(command: &[u8; 12], vector: &mut Vec<u8>) {
        println!("Function: other_message");
        let answer = match_command(command, vector);
        println!("{:?}", answer)
    }

    /// data_loaded is True if I'm waiting for a block
    /// get_data_vector is empty if I don't need any more blocks
    /// reading_headers si True if we're currently reading headers. Because it's so much faster than reading blocks, we prefer to read headers first and when finished, read blocks
    fn block_work(
        &mut self,
        get_data_vector: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
        other_vector_len: usize,
        write_block_stream: &TcpStream,
    ) {
        println!("Function: block_work");
        // println!("!data_loaded: {}",!*data_loaded);
        // println!("!get_data_vector.is_empty():{}",!get_data_vector.is_empty());
        // println!("!reading_headers:{}",*reading_headers);
        // if (!*data_loaded)&&!(get_data_vector.is_empty())&&(!*reading_headers){
        if get_data_vector.len() == other_vector_len {
            let filter = FilterLoadMessage::new(0, vec![], 0, 0, 0);
            let _result = filter.send(write_block_stream);
            //let mut get_data = InvOrGetDataMessage::new(CompactSize::from_usize_to_compact_size(0),vec![]);
            // si todo ok
            if let Some(origin) = get_data_vector.first() {
                let _result = origin.send_get_data(write_block_stream);
            }
        }
    }

    /// Reads all outputs and stores them in a hashmap. The tx hash is the key
    fn add_to_utxo(
        &mut self,
        utxo_set: &mut HashMap<[u8; 32], Vec<Output>>,
        block: &mut BlockMessage,
        txid: &mut Vec<[u8; 32]>,
        sender_to_interface: SenderInterface,
    ) {
        let mut  blocks = match self.blocks.lock(){
            Ok(v)=>v,
            Err(_v)=>return,
        };
        blocks.insert(header_calculate_doublehash_array_be(&block.get_block_header()).unwrap_or([0; 32]), block.serialize());
        let mut tx = match self.tx.lock(){
            Ok(v)=>v,
            Err(_v)=>return,
        };
        for id in txid.iter(){
            if tx.contains_key(id){
                tx.remove(id);
            };
        };
        let mut txs = block.get_tx();
        println!("tx actual {:?}", txs);
        while !txid.is_empty() {
            let copy_for_interface = txs[0].clone();
            if let Ok(mut txs) = self.interface_communicator.transactions.lock() {
                txs.insert(
                    u8_array_to_hex_string(&copy_for_interface.get_id()),
                    copy_for_interface.clone(),
                );
            }
            //Pertenece al usuario
            let belongs = self.is_user_tx(copy_for_interface.clone());
            let message = InterfaceMessages::AllTransactions(true, copy_for_interface, belongs);
            if sender_to_interface.send(message).is_ok() {}
            utxo_set.insert(txid.remove(0), txs.remove(0).get_output());
        }
        //println!("\nDiccionario en construccion {:?}", utxo_set);
    }

    /// Reads all outputs and stores them in a hashmap. The tx hash is the key
    /// -this version doesnot use the interface because the data is already store-
    fn add_to_utxo_available(
        utxo_set: &mut HashMap<[u8; 32], Vec<Output>>,
        block: &mut BlockMessage,
        txid: &mut Vec<[u8; 32]>,
    ) {
        let mut txs = block.get_tx();
        while !txid.is_empty() {
            //println!("tx = {:?}",txs[0]);
            utxo_set.insert(txid.remove(0), txs.remove(0).get_output());
        }
        //println!("\nDiccionario en construccion {:?}", utxo_set);
    }

    ///Updates a hash with all the Outputs used
    fn add_to_used_tx(used_tx: &mut HashMap<[u8; 32], Vec<u32>>, block: &mut BlockMessage) {
        let txs = block.get_tx();
        for i in txs {
            let inputs = i.get_input();
            for input in inputs {
                let outpoint = input.get_outpoint();
                //println!("id usada: {:?}",outpoint.get_hash());
                if let std::collections::hash_map::Entry::Vacant(e) =
                    used_tx.entry(outpoint.get_hash())
                {
                    e.insert(vec![outpoint.get_index()]);
                } else if let Some(x) = used_tx.get_mut(&outpoint.get_hash()) {
                    x.push(outpoint.get_index());
                }
            }
        }
    }

    ///remove the use outputs from the utxo set
    fn remove_use_tx(
        utxo_set: &mut HashMap<[u8; 32], Vec<Output>>,
        used_tx: &mut HashMap<[u8; 32], Vec<u32>>,
    ) {
        let mut keys_to_remove: Vec<[u8; 32]> = Vec::new();
        for (key, val) in used_tx.iter() {
            if utxo_set.contains_key(key) {
                //if !(key == &[170, 122, 132, 36, 60, 22, 153, 40, 201, 158, 225, 91, 142, 105, 156, 209, 218, 200, 57, 43, 111, 61, 199, 226, 157, 138, 153, 232, 99, 133, 24, 77]){
                if let Some(x) = utxo_set.get_mut(key) {
                    for i in val {
                        x[*i as usize] =
                            Output::new(0, CompactSize::from_usize_to_compact_size(0), vec![]);
                    }
                }
                keys_to_remove.push(*key);
            }
        }
        for key in keys_to_remove {
            if let Some(x) = used_tx.get_mut(&key) {
                *x = vec![];
            }
        }
        println!("diccionario modificado")
        //println!("\nDiccionario en construccion {:?}", utxo_set.keys());
    }

    /// searches for an address outputs
    fn create_address_utxo(&mut self, utxo_set: &mut MutexGuard<HashMap<[u8; 32], Vec<Output>>>) {
        self.utxo_collector.create_address_utxo(utxo_set);
        println!(
            "{:?}",
            self.utxo_collector
                .get_utxos()
                .get_key_value("mw2DzXinK8KaqunpYgjnGyCYcgHVb3SJWc")
        );
        println!("no more addreses");

        // Wallet
        self.interface_communicator
            .update_balance(&self.utxo_collector);
    }

    fn _outputs_for_tx(&mut self, amount: i64) -> Vec<(Outpoint, Output)> {
        let mut vector: Vec<(Outpoint, Output)> = vec![];

        let utxo_hash = &self.utxo_collector.get_utxos();

        for (_key, val) in utxo_hash.iter() {
            let mut balance = 0;
            for (i, j) in val {
                balance += j.get_value();
                vector.push((*i, j.clone()));
            }
            if balance > amount {
                break;
            }
        }
        vector
    }

    /// Initialize the necessary structures to launch the interface, returning the channels it will need for communication.
    fn start_interface(&mut self, peers: Vec<String>) -> StartInterfaceElements {
        let (sender_to_interface, receiver_from_node) =
            MainContext::channel(glib::PRIORITY_DEFAULT);
        let (sender_to_node, receiver_from_interface) =
            std::sync::mpsc::channel::<InterfaceMessages>();
        let mut handles_interface = Vec::new();
        let handle_interface = InterfaceHandler::start(sender_to_node, receiver_from_node);
        self.interface_communicator = InterfaceCommunicator::new();
        self.interface_communicator.start(
            peers,
            sender_to_interface.clone(),
            receiver_from_interface,
        );
        handles_interface.push(handle_interface);
        (handles_interface, sender_to_interface)
    }

    /// It returns the peers of the node.
    pub fn get_peers(&self) -> &Option<Vec<String>> {
        &self.peers
    }
}

#[cfg(test)]
mod bitnode_tests {
    use super::*;

    #[test]
    fn test_new_peers_is_none() {
        let node = BitcoinNode::new();
        assert!(node.peers.is_none());
    }

    #[test]
    fn test_new_blocks_is_none() {
        let node = BitcoinNode::new();
        assert!(node.blocks.lock().unwrap().is_empty());
    }
}
