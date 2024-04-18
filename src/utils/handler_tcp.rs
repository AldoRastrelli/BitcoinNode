use std::io::{Read, Write};
use std::net::TcpStream;

use std::thread;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub struct HandlePeer {
    pub tcp_stream: TcpStream,
    pub rx: Receiver<Vec<u8>>,
    pub tx: Sender<Vec<u8>>,
}

impl HandlePeer {
    pub fn new(ip: String) -> Result<HandlePeer, std::io::Error> {
        let ip_and_port: &str = &format!("{ip}:18333");
        let stream = TcpStream::connect(ip_and_port)?;

        let write_stream = stream.try_clone()?;
        let read_stream = stream.try_clone()?;

        let (tx_read, rx_read) = channel::<Vec<u8>>();
        let (tx_write, rx_write) = channel::<Vec<u8>>();

        let _handle_write = thread::spawn(move || {
            tcp_writer(write_stream, rx_write);
        });
        let _handle_read: JoinHandle<()> = thread::spawn(move || {
            tcp_reader(read_stream, tx_read);
        });

        Ok(HandlePeer {
            tcp_stream: stream,
            rx: rx_read,
            tx: tx_write,
        })
    }
}

/// Reads from a TcpStream and sends the data to a channel
pub fn tcp_reader(mut read_stream: TcpStream, tx: Sender<Vec<u8>>) {
    let mut buf = vec![0u8; 4096];
    loop {
        if !buf.is_empty() {
            let result = read_stream.read(&mut buf);

            if let Ok(n) = result {
                if n > 0 {
                    let response = &buf[..n];
                    if tx.send(response.to_vec()).is_ok() {}
                }
            }
        }
    }
}

/// Reads from a channel and writes to a TcpStream
pub fn tcp_writer(mut write_stream: TcpStream, rx: Receiver<Vec<u8>>) {
    loop {
        if let Ok(message) = rx.try_recv() {
            let _result = write_stream.write_all(&message);
            {}
        }
    }
}
