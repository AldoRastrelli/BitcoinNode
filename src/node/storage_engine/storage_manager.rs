use super::file_lines::FileLines;
use crate::message_structs::block_headers::BlockHeader;
use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::merkel_block::MerkleBlock;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/// Handles the storage of headers, blocks, transactions and logs
pub struct StorageManager {
    path: String,
    file: File,
}

impl StorageManager {
    // Creation

    /// Creates a new StorageManager for headers
    pub fn new_header_storage(is_client: bool) -> StorageManager {
        let mut path = "./storage/headers";
        if is_client {
            path = "./storage/headers_client"
        }
        let file = match File::options()
            .write(true)
            .append(true)
            .open(path)
        {
            Ok(file) => file,
            Err(_e) => {
                println!("Error opening file");
                std::process::exit(1); // O maneja el error de alguna otra manera
            }
        };

        StorageManager {
            path: path.to_string(),
            file,
        }
    }

    /// Creates a new StorageManager for blocks
    pub fn new_blocks_storage(is_client: bool) -> StorageManager {
        let mut path = "./storage/blocks";
        if is_client {
            path = "./storage/blocks_client"
        }
        let file = match File::options()
            .write(true)
            .append(true)
            .open(path)
        {
            Ok(file) => file,
            Err(_e) => {
                println!("Error opening file");
                std::process::exit(1); // O maneja el error de alguna otra manera
            }
        };

        StorageManager {
            path: path.to_string(),
            file,
        }
    }

    /// Creates a new StorageManager for merkle blocks
    pub fn new_merkle_storage(is_client: bool) -> StorageManager {
        let mut path = "./storage/merkles";
        if is_client {
            path = "./storage/merkles_client"
        }
        let file = match File::options()
            .write(true)
            .append(true)
            .open(path)
        {
            Ok(file) => file,
            Err(_e) => {
                println!("Error opening file");
                std::process::exit(1); // O maneja el error de alguna otra manera
            }
        };

        StorageManager {
            path: path.to_string(),
            file,
        }
    }

    /// Creates a new StorageManager for logs
    pub fn new_log_storage() -> StorageManager {
        let path = "./logs/log.txt".to_string();
        let file = match File::options()
            .write(true)
            .append(true)
            .open("./logs/log.txt")
        {
            Ok(file) => file,
            Err(_e) => {
                println!("Error opening file");
                std::process::exit(1); // O maneja el error de alguna otra manera
            }
        };

        StorageManager {
            path,
            file,
        }
    }

    /// Creates a new StorageManager for testing
    pub fn new_testing_storage() -> Result<StorageManager, Box<dyn Error>> {
        let path = "./storage/testing".to_string();
        let file = match File::options()
            .write(true)
            .append(true)
            .open(path.clone())
        {
            Ok(file) => file,
            Err(_e) => {
                println!("Error opening file");
                std::process::exit(1); // O maneja el error de alguna otra manera
            }
        };

        let sm = StorageManager {
            path,
            file,
        };

        _ = sm.clean_file();

        Ok(sm)
    }

    // File handling

    /// Cleans the file
    pub(self) fn clean_file(&self) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(&self.path)?;
        file.write_all(&[])?;

        Ok(())
    }

    /// Cleans the file and resets the headers_hash
    pub fn reset(&mut self) {
        _ = self.clean_file();
    }

    /// Returns true if the file is empty
    pub fn file_is_empty(&self) -> Result<bool, Box<dyn Error>> {
        let path = &self.path;
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(reader.lines().next().is_none())
    }

    // Read and Write

    /// Saves a Vectors of Strings into a single line in the CSV file
    /// #Errors
    /// Returns an error if the file cannot be opened
    fn writeln_to_csv(&mut self, data: Vec<String>) -> Result<(), Box<dyn Error>> {
        for line in data {
            self.file.write_all(line.as_bytes())?;
            self.file.write_all(b",")?; // Add a comma delimiter between fields
        }

        self.file.write_all(b"\n")?; // Add a new line after each line of data
        match self.file.flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Reads the CSV file and returns a vector of vectors of Strings
    pub fn read_csv_file(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path: &String = &self.path;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut all_fields = Vec::new();
        // Iterate over each line in the CSV file
        for (_, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            all_fields.push(fields);
        }
        Ok(all_fields)
    }

    /// Returns a FileLines struct, which is an iterator over the lines of the CSV file
    pub fn get_file_reader(&self) -> Result<FileLines, Box<dyn Error>> {
        let path = &self.path;
        println!("Path: {}", path);
        match FileLines::new(path) {
            Ok(file) => Ok(file),
            Err(_e) => {
                println!("Error opening file");
                Err("Error opening file".into())
            }
        }
    }

    // Headers

    /// Receives a BlockHeader and stores its headers in the CSV file
    pub fn store_header(&mut self, header: BlockHeader) -> bool {
        match self.save_header_data_to_file(header) {
            Ok(_v) => {
                //println!("Header {:?} written correctly", header);
                true
            }
            Err(_e) => {
                println!("Error writing header");
                false
            }
        }
    }

    /// Saves a BlockHeader
    /// #Errors
    /// Returns an error if the file cannot be opened
    pub fn save_header_data_to_file(&mut self, header: BlockHeader) -> Result<(), Box<dyn Error>> {
        let csv = header.serialize();
        let string = csv.iter().map(|&byte| byte.to_string()).collect();
        self.writeln_to_csv(string)?;

        Ok(())
    }

    // Blocks

    /// Receives a BlockMessage and stores its headers in the CSV file
    pub fn store_block(&mut self, block: BlockMessage) -> bool {
        match self.save_block_data_to_file(block) {
            Ok(_v) => {
                //println!("Block {:?} written correctly", block);
                true
            }
            Err(_e) => {
                println!("Error writing header");
                false
            }
        }
    }

    /// Saves a BlockMessage into the file
    pub fn save_block_data_to_file(&mut self, block: BlockMessage) -> Result<(), Box<dyn Error>> {
        let csv = block.serialize();
        let string = csv.iter().map(|&byte| byte.to_string()).collect();
        self.writeln_to_csv(string)?;
        Ok(())
    }

    /// Saves a MerkleBlock into the file
    pub fn save_merkle_data_to_file(&mut self, block: MerkleBlock) -> Result<(), Box<dyn Error>> {
        let csv = block.serialize();
        let string = csv.iter().map(|&byte| byte.to_string()).collect();
        self.writeln_to_csv(string)?;
        Ok(())
    }

    // Logs

    /// Casts a tuple of String and Vec<u8> into a Vec<String>
    fn tuple_to_vector(data: (String, Vec<u8>)) -> Vec<String> {
        let mut vector = Vec::new();
        vector.push(data.0);
        let mut message = String::new();
        for byte in data.1.iter() {
            message.push_str(&format!("{:02X}", byte));
        }
        vector.push(message);
        vector
    }

    /// Stores a log in the CSV file
    pub fn store_log(&mut self, data: (String, Vec<u8>)) {
        let new_line = Self::tuple_to_vector(data);
        if self.writeln_to_csv(new_line).is_ok() {
            // println!("Log written correctly");
        } else {
            println!("Error writing log");
        }
    }
}

#[cfg(test)]

mod store_manager_tests {

    use std::sync::Arc;
    use std::sync::Mutex;

    use crate::message_structs::compact_size::CompactSize;

    use super::*;

    fn _setup() -> Arc<Mutex<StorageManager>> {
        let storage_manager = StorageManager::new_testing_storage().unwrap();
        Arc::new(Mutex::new(storage_manager))
    }

    //Header Tests

    //#[test]
    fn _test_save_header_data_to_file() {
        let binding = _setup();
        let mut lock = binding.lock();

        while lock.is_err() {
            lock = binding.lock();
        }
        let mut storage_manager = lock.unwrap();
        storage_manager.reset();

        let header = BlockHeader {
            version: 1,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 12332423,
            n_bits: 0,
            nonce: 0,
        };
        let _result = storage_manager.save_header_data_to_file(header.clone());

        let mut reader = storage_manager.get_file_reader().unwrap();
        let mut line_read = reader.next_line().unwrap();

        println!("Line read: {:?}", line_read);

        let deserialized_header: BlockHeader = BlockHeader::deserialize(&mut line_read).unwrap();

        assert_eq!(deserialized_header, header);
        storage_manager.reset();
    }

    //Block Tests

    //#[test]
    fn _test_save_block_data_to_file() {
        let binding = _setup();
        let mut lock = binding.lock();

        while lock.is_err() {
            lock = binding.lock();
        }
        let mut storage_manager = lock.unwrap();
        storage_manager.reset();

        let header = BlockHeader {
            version: 1,
            previous_block_header_hash: [0; 32],
            merkle_root_hash: [0; 32],
            time: 12332423,
            n_bits: 0,
            nonce: 0,
        };
        let block = BlockMessage {
            block_header: header,
            tx_count: CompactSize {
                prefix: 0,
                number_vec: vec![0],
                number: 0,
            },
            transaction_history: Vec::new(),
        };
        let _result = storage_manager.save_block_data_to_file(block.clone());

        let mut reader = storage_manager.get_file_reader().unwrap();
        let mut line_read = reader.next_line().unwrap();

        println!("Line read: {:?}", line_read);

        let deserialized_block: BlockMessage = BlockMessage::deserialize(&mut line_read).unwrap();

        assert_eq!(deserialized_block, block);
        storage_manager.reset();
    }
}
