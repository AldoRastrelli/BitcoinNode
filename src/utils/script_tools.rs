use bitcoin_hashes::{ripemd160, sha256, sha256d, Hash};
use bs58;

///Adds a an i number of items to a stack
///modify the start of last_addition
///return the element added to the stack
fn add_to_stack(
    stack: &mut Vec<u8>,
    i: u8,
    script: &mut Vec<u8>,
    start_last_addition: &mut usize,
) -> Vec<u8> {
    let binding = script.drain(..(i as usize)).collect::<Vec<u8>>();
    *start_last_addition = stack.len();
    stack.extend_from_slice(&binding);
    binding
}

///For the script of an input it identifies the sig an key
pub fn get_sig_and_pubkey(script: &[u8]) -> Vec<Vec<u8>> {
    let mut vector = vec![vec![]];
    let mut stack = Vec::new();
    let mut start_last_addition = 0;
    let mut clone_sript = script.to_owned();
    while !clone_sript.is_empty() {
        let i = clone_sript.remove(0);
        if i < 76 && i < clone_sript.len() as u8 {
            vector.push(add_to_stack(
                &mut stack,
                i,
                &mut clone_sript,
                &mut start_last_addition,
            ));
        }
    }
    vector
}

///It applys the hash160 to an stack an returns the hash as a vec
fn op_hash160(stack: &mut [u8]) -> Vec<u8> {
    let hash = sha256::Hash::hash(stack);
    let rip = ripemd160::Hash::hash(&hash.to_byte_array());
    rip.to_byte_array().to_vec()
}

///calculates the checksum
fn calculate_checksum(payload: &[u8]) -> [u8; 4] {
    let hash = sha256d::Hash::hash(payload);
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&hash[..4]);
    checksum
}

///adds the information needed to tranform the pubKey hash into an address
fn from_pubkey_hash_to_address(stack: Vec<u8>) -> Vec<u8> {
    let mut addres = vec![0x6f];
    addres.extend_from_slice(&stack);
    let checksum = calculate_checksum(&addres);
    addres.extend_from_slice(&checksum);
    addres
}

///Creates the address from an script input an returns a vec
fn bitcoin_address_input(script: &[u8]) -> Vec<u8> {
    let mut stack = get_sig_and_pubkey(script);
    let mut pubkey_hash = vec![];
    if stack.len() > 1 {
        pubkey_hash = op_hash160(&mut stack[1]);
    }
    let mut address = vec![];
    if !pubkey_hash.is_empty() {
        address = from_pubkey_hash_to_address(pubkey_hash);
    }
    address
}

/// turns the addres from a vec to a string with b58
pub fn bitcoin_address_in_b58_input(script: &[u8]) -> String {
    let vector = bitcoin_address_input(script);
    let addres = bs58::encode(vector).into_string();
    addres
}

///Copy the Top item on the stack
fn op_dup(
    stack: &mut Vec<u8>,
    last_addition: &mut [u8],
    start_last_addition: &mut usize,
) -> Vec<u8> {
    *start_last_addition = stack.len();
    stack.extend_from_slice(last_addition);
    last_addition.to_vec()
}

///It applys the hash160 to an stack an returns the hash as a vec but alse modify the stack with the hash
/// and its eliminates the unhash value
fn op_hash160_script(
    stack: &mut Vec<u8>,
    last_addition: &mut [u8],
    start_last_addition: &mut usize,
) -> Vec<u8> {
    let hash = sha256d::Hash::hash(last_addition);
    let rip = ripemd160::Hash::hash(&hash.to_byte_array());
    stack.drain(*start_last_addition..(stack.len() - 1));
    stack.extend_from_slice(&rip.to_byte_array());
    rip.to_byte_array().to_vec()
}

///checks if two top items of the stack are equals
/// removes both from teh stack and adds 1 or 0 depending on if they are equal
fn op_equal(stack: &mut Vec<u8>, start_last_addition: &mut usize) -> bool {
    let mut bool = false;
    let len = stack.len() - *start_last_addition;
    let x = stack
        .drain(*start_last_addition..(stack.len() - 1))
        .collect::<Vec<u8>>();
    let y = stack
        .drain(stack.len() - len..stack.len() - 1)
        .collect::<Vec<u8>>();
    *start_last_addition = stack.len();
    if x == y {
        stack.push(1);
        bool = true;
    } else {
        stack.push(0);
    };
    bool
}

/// works with the script
pub fn script_work(script: &[u8], pubkey: Vec<u8>) -> bool {
    let mut owner = false;
    let mut stack = Vec::new();
    let mut last_addition = Vec::new();
    //let sig =;
    let mut start_last_addition = 0;
    stack.extend_from_slice(&pubkey);
    last_addition.extend_from_slice(&pubkey);
    let mut clone_sript = script.to_owned();
    while !clone_sript.is_empty() {
        let i = clone_sript.remove(0);
        if i < 76 {
            last_addition = add_to_stack(&mut stack, i, &mut clone_sript, &mut start_last_addition);
        } else if i == 118 {
            last_addition = op_dup(&mut stack, &mut last_addition, &mut start_last_addition);
        } else if i == 169 {
            last_addition =
                op_hash160_script(&mut stack, &mut last_addition, &mut start_last_addition);
        } else if i == 135 {
            owner = op_equal(&mut stack, &mut start_last_addition);
        } else if i == 136 {
            owner = op_equal(&mut stack, &mut start_last_addition);
            if !owner {
                return owner;
            };
            stack.pop();
        }
        // else if i ==172 {
        //     Self::op_checksig(&mut stack,sig)
        // }
    }
    owner
}

///Searches for the address in the output script
/// return a vector
fn bitcoin_address_output(script: &[u8]) -> Vec<u8> {
    let mut stack = Vec::new();
    let mut start_last_addition = 0;
    let mut clone_sript = script.to_owned();
    while !clone_sript.is_empty() {
        let i = clone_sript.remove(0);
        if i < 76 && i < clone_sript.len() as u8 {
            add_to_stack(&mut stack, i, &mut clone_sript, &mut start_last_addition);
        }
    }
    if !stack.is_empty() {
        stack = from_pubkey_hash_to_address(stack);
    }
    stack
}

/// turns the addres from a vec to a string with b58
#[must_use] pub fn bitcoin_address_in_b58_output(script: &[u8]) -> String {
    let vector = bitcoin_address_output(script);
    let addres = bs58::encode(vector).into_string();
    addres
}

/// Returns the address in base58
/// # Errors
/// If it could not be decoded
pub fn from_adderss_to_vec(address: &str) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(address).into_vec()
}
