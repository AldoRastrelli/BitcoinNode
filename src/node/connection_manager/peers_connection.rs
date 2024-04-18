use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, MutexGuard};
use std::thread;

use std::io::Read;
use std::net::TcpStream;

use crate::message_structs::bitcoin_message_header::BitcoinMessageHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::compact_size::CompactSize;
use crate::message_structs::get_headers_message::GetHeadersMessage;
use crate::message_structs::headers_message::HeadersMessage;
use crate::message_structs::inv_or_get_data_message::InvOrGetDataMessage;
use crate::message_structs::version_message::VersionMessage;
use crate::utils::commands::{get_type, MessageType};
use crate::utils::thread_pool;
use glib::Sender as InterfaceSender;

use crate::node::interface::interface_communicator::InterfaceMessages;
use crate::node::storage_engine::storage_manager::StorageManager;
use crate::node::validation_engine::validations::verify_header;
use crate::utils::build_messages::build_version_message;
use crate::utils::build_messages::get_magic_bytes;

use std::sync::Mutex;
use std::thread::JoinHandle;
type SenderLogger = Arc<Mutex<Sender<(String, Vec<u8>)>>>;
type Storage = Arc<Mutex<StorageManager>>;
type SenderToInterface = InterfaceSender<InterfaceMessages>;

/// Extracts the Headers from a HeadersMessage and validates them. If valid, it stores them and pushes them to the InterfaceChannel.
pub fn get_all_headers(
    read_stream: &TcpStream,
    mut message: Vec<u8>,
    sender_to_interface: SenderToInterface,
    storage_manager: &Storage,
    blocks_to_read: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
    merkel_to_read: &mut MutexGuard<Vec<InvOrGetDataMessage>>,
    last_header: &Arc<Mutex<[u8; 32]>>,
)->Option<HeadersMessage>{
    let headers = match HeadersMessage::deserialize(&mut message) {
        Ok(v) => v,
        Err(_e) => return None,
    };
    let mut hash = match last_header.lock() {
        Ok(v) => v,
        Err(e) => {
            println!("error:{:?}", e);
            return None
        }
    };
    let last_hash = headers.last_hash();
    if last_hash != [0; 32] && last_hash != *hash {
        let get_headers = GetHeadersMessage {
            version: 70015,
            hash_count: CompactSize {
                prefix: 0,
                number_vec: vec![1],
                number: 1,
            },
            block_locator_hashes: vec![last_hash],
            hash_stop: last_hash,
        };
        let _result = get_headers.send(read_stream);
        println!("\n enviado {:?} \n", get_headers);

        //*last_header.get_mut().unwrap()=headers.last_hash();
        *hash = last_hash;
        println!("cambio : {:?}", hash);

        let mut get_data_block = headers.get_data_with_type(2);
        if !get_data_block.is_empty() {
            blocks_to_read.append(&mut get_data_block);
        }
        blocks_to_read.dedup();
        let mut get_merkel_block = headers.get_data_with_type(3);
        if !get_merkel_block.is_empty() {
            merkel_to_read.append(&mut get_merkel_block);
        }
        merkel_to_read.dedup();
        // let mut get_data_merkel = headers.get_data_with_type(3);
        // if !get_data_merkel.is_empty() {
        //     merkel_to_read.append(&mut get_data_merkel);
        // }

        let thread_pool = thread_pool::ThreadPool::new(30);

        // Validations
        for h in headers.headers.iter() {
            let sender_to_interface = sender_to_interface.clone();
            let storage_manager = storage_manager.clone();
            let h = h.clone();

            thread_pool.execute(move || {
                let header_is_valid = verify_header(&h);
                if !header_is_valid {
                    println!("Header is not valid");
                    return;
                }
                println!("Header is valid");
                let mut storage_manager_lock = storage_manager.lock();

                while storage_manager_lock.is_err() {
                    storage_manager_lock = storage_manager.lock();
                }

                match storage_manager_lock {
                    Ok(mut sm) => {
                        if sm.store_header(h.clone()) {
                            let message = InterfaceMessages::DebugHeaders(h.clone());
                            if sender_to_interface.send(message).is_ok() {}
                        }
                    }
                    Err(e) => {
                        println!("Error locking storage_manager: {:?}", e);
                    }
                }
            });
        }

        println!("\n------Validation Threads finished successfully------\n");
        Some(headers)
    }
    else{
        None
    }
}

/// Receives a BlockMessage and deserializes it, and sends it to the Interface Channel
pub fn get_blocks(sender: Arc<Mutex<Sender<BlockMessage>>>, message: &mut Vec<u8>) {
    let block = match BlockMessage::deserialize(message) {
        Ok(v) => v,
        Err(_e) => return,
    };
    if let Ok(s) = sender.lock() {
        if s.send(block).is_ok() {}
    }
}

/// Saves the received blocks
pub fn save_blocks(
    receiver: Arc<Mutex<Receiver<BlockMessage>>>,
    sender_interface: SenderToInterface,
    blocks: Arc<Mutex<HashMap<String, BlockMessage>>>,
    total_blocks: Arc<Mutex<(usize, bool)>>,
    is_client: bool
) -> JoinHandle<()> {
    let mut storage_manager_blocks = StorageManager::new_blocks_storage(is_client);

    thread::spawn(move || {
        if let Ok(message_mutex) = receiver.lock() {
            let mut id_block = 0;
            loop {
                if let Ok(block) = message_mutex.recv() {
                    if storage_manager_blocks.store_block(block.clone()) {
                        if let Ok(total_blocks) = total_blocks.lock() {
                            let message = InterfaceMessages::DebugBlocks(id_block, block.clone(), total_blocks.0);
                            if sender_interface.send(message).is_ok() {
                                if let Ok(mut b) = blocks.lock() {
                                    b.insert(id_block.to_string(), block);
                                }
                                id_block += 1;
                            }
                        }
                    }
                }
            }
        }
    })
}

/// It attempts to deserialize the payload into a Header. If it succeeds, it calls read_exact to access the data in the message.
fn is_message(
    payload: &mut Vec<u8>,
    mut read_stream: &TcpStream,
    tx: &Sender<Vec<u8>>,
) -> Result<&'static str, &'static str> {
    let (message, _payload) = match BitcoinMessageHeader::deserialize(payload) {
        Ok(v) => v,
        Err(_e) => return Err("error en el lectura bitcoin header"),
    };
    println!("\nrecibido bitcoin header:{:?}", message);
    let mut full_message: Vec<u8> = Vec::new();
    full_message.extend_from_slice(&message.command());
    let missing_data = message.payload() as usize;
    if missing_data > 0 {
        let mut buf = vec![0u8; missing_data];
        match read_stream.read_exact(&mut buf) {
            Ok(r) => r,
            Err(_) => return Err("error en el buff del mensaje"),
        };
        full_message.extend_from_slice(&buf);
    } else {
        full_message.extend_from_slice(&message.serialize());
    }
    let Ok(_result) = tx.send(full_message) else {
        return Err("error en el send");
    };
    Ok("mensaje recibido")
}

pub fn deserialize_message_from_client(
    buffer: &mut Vec<u8>,
    mut read_stream: &TcpStream,
) -> (MessageType,Vec<u8>) {
    let (message, _payload) = match BitcoinMessageHeader::deserialize(buffer) {
        Ok(v) => v,
        Err(_e) => return (MessageType::Unknown,vec![]),
    };
    println!("\nrecibido bitcoin header:{:?}", message);
    let mut full_message: Vec<u8> = Vec::new();
    full_message.extend_from_slice(&message.command());
    let missing_data = message.payload() as usize;
    if missing_data > 0 {
        let mut buf = vec![0u8; missing_data];
        match read_stream.read_exact(&mut buf) {
            Ok(r) => r,
            Err(_) => return (MessageType::Unknown,vec![]),
        };
        full_message.extend_from_slice(&buf);
    } else {
        full_message.extend_from_slice(&message.serialize());
    }
    let command: [u8; 12] = match full_message.drain(0..12).collect::<Vec<u8>>().try_into() {
        Ok(a) => a,
        Err(_) => panic!("Failed to convert vector to array"),
    };
    (get_type(&command),full_message)
}

/// Reads the stream and sends the received messages to the interface
pub fn reader(
    cloned_stop_flag: Arc<Mutex<bool>>,
    mut read_stream: TcpStream,
    tx: Sender<Vec<u8>>,
    logger_channel: SenderLogger,
    receiver: Receiver<u8>,
) {
    loop {
        let mut lock_stop_flag = cloned_stop_flag.lock();
        while lock_stop_flag.is_err() {
            lock_stop_flag = cloned_stop_flag.lock();
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
        let confirmation = match receiver.recv() {
            Ok(v) => v,
            Err(v) => {
                println!("Error receiving message with recv, {:?}", v);
                let mut lock_stop_flag = cloned_stop_flag.lock();
                while lock_stop_flag.is_err() {
                    println!("Trying to obtain lock_stop_flag lock");
                    lock_stop_flag = cloned_stop_flag.lock();
                }
                println!("Obtained");
                match lock_stop_flag {
                    Ok(mut stop_flag) => {
                        println!("stop_flag = true");
                        *stop_flag = true;
                    }
                    Err(e) => {
                        println!("Error locking stop_flag: {:?}", e);
                    }
                }
                break;
            }
        };
        if confirmation == 0 {
            return;
        }

        let mut buf = vec![0u8; 24];

        let mut reading_status = read_stream.read_exact(&mut buf);
        let mut x = 0;
        while reading_status.is_err() && x < 1000 {
            reading_status = read_stream.read_exact(&mut buf);
            x += 1;
        }
        // If error, break
        if reading_status.is_err() {
            println!("Error reading stream with read_exact: {:?}", reading_status);
            match read_stream.read(&mut buf) {
                Ok(v) => {
                    println!("Read: {:?}", v);
                    if v == 0 {
                        break;
                    }
                }
                Err(e) => {
                    let _result = tx.send(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
                    println!("Error reading stream with read: {:?}", e);
                    break;
                }
            };
        } else {
            let response = &buf;
            let mut payload = response.to_vec();

            let clone_message = payload.clone();
            if let Ok(stream) = read_stream.try_clone() {
                if let Ok(addr) = stream.peer_addr() {
                    if let Ok(s) = logger_channel.lock() {
                        if s.send((addr.ip().to_string(), clone_message)).is_ok() {}
                    }
                }
            }
            // If there are not magic bytes present, continue
            if [payload[0], payload[1], payload[2], payload[3]] == *get_magic_bytes() {
                match is_message(&mut payload, &read_stream, &tx) {
                    Ok(v) => println!("{}", v),
                    Err(v) => {
                        println!("Error: with the payload {}", v);
                        return;
                    }
                };
            }
        }
    }
}

/// Sends the version message and waits for the verack message. Then, it builds the sendHeaders
pub fn writer(write_stream: &TcpStream) {
    
    handshake(write_stream);

    //Build sendHeaders
    let send_headers = BitcoinMessageHeader::send_headers();
    let _result = send_headers.send(write_stream);
    println!("\n enviado {:?} \n", send_headers);
}

pub fn handshake(write_stream: &TcpStream) {
    send_version(write_stream);
    send_verack(write_stream);
}

pub fn send_version(write_stream: &TcpStream) {
    // Build version message
    let mut header = match build_version_message(write_stream) {
        Ok(v) => v,
        Err(_e) => return,
    };

    let (_message_header, payload) = match BitcoinMessageHeader::deserialize(&mut header) {
        Ok(v) => v,
        Err(_e) => return,
    };

    let version = match VersionMessage::deserialize(payload) {
        Ok(v) => v,
        Err(_e) => return,
    };

    // Send version message
    let _result = version.send(write_stream);
    println!("\n enviado {:?} \n", version);
}

pub fn send_verack(write_stream: &TcpStream) {
    // Build verack message
    let verack = BitcoinMessageHeader::verack();

    // Send verack message
    let _result = verack.send(write_stream);
    println!("\n enviado {:?} \n", verack);
}

/// Sends the getheaders message
pub fn get_headers(write_stream: &mut TcpStream, last_header: &Arc<Mutex<[u8; 32]>>) {
    //Build get_headers message
    let hash = match last_header.lock() {
        Ok(v) => v,
        Err(e) => {
            println!("error:{:?}", e);
            return;
        }
    };
    let get_headers = GetHeadersMessage {
        version: 70015,
        hash_count: CompactSize {
            prefix: 0,
            number_vec: vec![1],
            number: 1,
        },
        block_locator_hashes: vec![*hash],
        hash_stop: *hash,
    };
    // let get_headers_srt = match GetHeadersMessage::build_default() {
    //     Ok(v) => v,
    //     Err(_e) => return,
    // };

    let _result = get_headers.send(write_stream);
    println!("\n enviado {:?} \n", get_headers);
}