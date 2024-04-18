use std::sync::mpsc::{Receiver, Sender};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use super::configs::config::get_logger_file_location;

type SenderLogger = Arc<Mutex<Sender<(String, Vec<u8>)>>>;
type ReceiverLogger = Arc<Mutex<Receiver<(String, Vec<u8>)>>>;

pub struct Logger {
    pub sender: SenderLogger,
    pub receiver: ReceiverLogger,
}

impl Default for Logger {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<(String, Vec<u8>)>();
        let receiver_mutex = Arc::new(Mutex::new(receiver));
        let sender_mutex = Arc::new(Mutex::new(sender));
        Logger {
            sender: sender_mutex,
            receiver: receiver_mutex,
        }
    }
}

impl Logger {
    fn format_message(data: (String, Vec<u8>)) -> String {
        let message = data
            .1
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect::<String>();
        format!("{}, {:?}", data.0, message)
    }

    fn open_file() -> Option<File> {
        let path = match get_logger_file_location() {
            Ok(path) => path,
            Err(error) => {
                println!("Error: {}", error);
                // Default value
                "./logs/log.txt".to_string()
            }
        };
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.as_str())
            .ok()?;
        Some(file)
    }

    pub fn start(&self) -> JoinHandle<()> {
        if let Some(mut file) = Self::open_file() {
            let receiver = Arc::clone(&self.receiver);
            thread::spawn(move || {
                if let Ok(message_mutex) = receiver.lock() {
                    loop {
                        if let Ok(message) = message_mutex.recv() {
                            let formatted_message = Self::format_message(message);

                            if file
                                .write_all(format!("{}\n", formatted_message).as_bytes())
                                .is_err()
                            {
                                println!("Error escribiendo en el archivo");
                            }
                        }
                    }
                }
            })
        } else {
            thread::Builder::new()
                .spawn(|| {})
                .expect("Error al crear el hilo")
        }
    }

    pub fn get_sender_clone(&self) -> SenderLogger {
        self.sender.clone()
    }
}
