use std::{
    error::Error,
    net::{IpAddr, TcpStream},
};

use rand::Rng;

use crate::utils::configs::config::get_protocol_version;

use bitcoin_hashes::{sha256d, Hash};
use std::time::{SystemTime, UNIX_EPOCH};

/// This function returns the magic bytes of the protocol
pub fn get_magic_bytes() -> &'static [u8; 4] {
    &[0x0b, 0x11, 0x09, 0x07]
}

/// Builds a Header Message from a command and a payload
pub fn build_header_message(command: &str, payload: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut message = Vec::new();

    let magic_bytes = get_magic_bytes();
    message.extend(magic_bytes);

    let command_bytes = command.as_bytes(); // current command as bytes

    let mut command_name = Vec::new();
    command_name.extend_from_slice(command_bytes);
    command_name.extend(vec![0; 12 - command_bytes.len()]); // add padding
    message.extend(&command_name);

    message.extend(&(payload.len() as u32).to_le_bytes()); // payload length

    let checksum = &sha256d::Hash::hash(&payload)[..4];
    let mut checksum_le = [0u8; 4];
    checksum_le.copy_from_slice(checksum);
    checksum_le.reverse();
    message.extend(checksum); // checksum - payload hash first 4 bytes

    message.extend(&payload);

    Ok(message)
}
/// Builds a version message for a stream
pub fn build_version_message(stream: &TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut payload: Vec<u8> = Vec::new();
    let protocol_version = get_protocol_version()?;

    payload.extend(&protocol_version.to_le_bytes()); // protocol version

    payload.extend(&1u64.to_le_bytes()); // network services of sender

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let timestamp_le = timestamp.to_le_bytes();

    payload.extend(timestamp_le); // timestamp

    payload.extend(&0u64.to_le_bytes()); // network services of receiver

    let ipv6_peer_addr = match stream.peer_addr()?.ip() {
        IpAddr::V4(ipv4_peer_addr) => ipv4_peer_addr.to_ipv6_mapped(),
        IpAddr::V6(ipv6_peer_addr) => ipv6_peer_addr,
    };

    let ipv6_peer_num = u128::from_be_bytes(ipv6_peer_addr.octets());
    let ipv6_peer_num_bytes = ipv6_peer_num.to_be_bytes();

    payload.extend(ipv6_peer_num_bytes); // peer ip

    let peer_port = stream.peer_addr()?.port().to_be_bytes();

    payload.extend(peer_port); // peer port

    payload.extend(&0u64.to_le_bytes()); // network services of receiver

    let ipv6_local_addr = match stream.local_addr()?.ip() {
        IpAddr::V4(ipv4_local_addr) => ipv4_local_addr.to_ipv6_mapped(),
        IpAddr::V6(ipv6_local_addr) => ipv6_local_addr,
    };

    let ipv6_local_num = u128::from_be_bytes(ipv6_local_addr.octets());
    let ipv6_local_num_bytes = ipv6_local_num.to_be_bytes();
    payload.extend(ipv6_local_num_bytes); // local ip

    let local_port = stream.local_addr()?.port().to_be_bytes();
    payload.extend(local_port); // local port

    let mut rng = rand::thread_rng();
    let nonce: u64 = rng.gen();
    let nonce_bytes = nonce.to_le_bytes();

    payload.extend(nonce_bytes);
    payload.extend(&0_u32.to_le_bytes());

    let user_agent = b"/Rusteze:0.1.0/";

    payload.extend(&(user_agent.len()).to_le_bytes());
    payload.extend(user_agent);

    let message = build_header_message("version", payload)?;

    Ok(message)
}

#[cfg(test)]

mod build_messages_tests {

    use super::*;

    #[test]
    fn test_build_version_message() {
        let peer = "104.155.226.24:18333";

        let stream = TcpStream::connect(peer).unwrap();
        let message = build_version_message(&stream);

        assert!(message.is_ok())
    }

    #[test]
    fn test_build_header_message() {}
}
