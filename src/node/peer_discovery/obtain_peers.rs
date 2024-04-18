use rand::seq::SliceRandom;
use std::{error::Error, net::ToSocketAddrs};

use crate::utils::configs::config::{get_dns, get_testnet_port};

/// It obtains the peers from the DNS.
/// It returns a vector of Strings containing the peer's IP addresses.
/// #Errors
/// - It returns an error if the testnet port is invalid.
/// - It returns an error if the DNS is invalid.
/// - It returns an error if the DNS lookup fails.
pub fn obtain_peers(client_address: Option<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut seeds: Vec<String> = Vec::new();
    let testnet_port: u16;
    let dns: Result<String, Box<dyn Error>>;

    match get_testnet_port() {
        Ok(port) => {
            testnet_port = port;
            dns = get_dns();
        }
        Err(_) => {
            return Err("Invalid Testnet Port".into());
        }
    }

    let lookup = match dns {
        Ok(dns) => (dns, testnet_port).to_socket_addrs(),
        Err(_) => {
            return Err("Invalid DNS".into());
        }
    };

    match lookup {
        Ok(lookup) => {
            for host in lookup {
                seeds.push(host.to_string());
            }
        }
        Err(_) => {
            return Err("DNS lookup failed".into());
        }
    }
    seeds.shuffle(&mut rand::thread_rng());
    if let Some(address) = client_address {
        seeds.insert(0, address)
    }
    println!("Seeds: {:?}", seeds);
    Ok(seeds)
}

#[cfg(test)]
mod obtain_peers_tests {
    use super::*;

    #[test]
    fn test_peer_discovery() {
        let client_address:Option<String> = None;
        let ips = obtain_peers(client_address).unwrap();
        assert!(!ips.is_empty());
    }
}
