use crate::message_structs::block_message::BlockMessage;
use crate::message_structs::merkel_block::MerkleBlock;
use crate::message_structs::tx_message::TXMessage;
use crate::utils::array_tools::{
    cast_vec_to_binary_vec, reverse_array, u8_array_to_hex_string,
};
use crate::utils::queue::Queue;
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use crate::node::validation_engine::hashes::vec_calculate_doublehash_array_be;
use crate::node::validation_engine::merkles::merkle_node::*;

#[derive(Debug)]
/// Merkle Tree structure: Binary Tree that completes itself by level
pub struct MerkleTree {
    root: OptionalMerkleRoot,
    count: usize,
    hashes_count: usize,
    hashes: Vec<[u8; 32]>,
}

impl Default for MerkleTree {
    fn default() -> Self {
        MerkleTree::new()
    }
}

impl MerkleTree {
    /// Initializes an empty Merkle Tree
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            count: 0,
            hashes_count: 0,
            hashes: Vec::new(),
        }
    }

    /// Builds an empty Merkle Tree with a given number of txids.
    /// This means the leafs are going to be the same number as the txid_num and the tree will be build keeping the txid_nodes at the leafs.
    pub fn build_empty_tree(txid_num: usize) -> Self {
        let mut tree = MerkleTree::new();

        if txid_num == 1 {
            for _ in 0..2 {
                tree._insert_hash([0; 32]);
            }
            tree._set_txid_nodes();
            return tree;
        }

        let max_height = Self::_get_max_height(txid_num);

        let mut nodes = Self::_get_first_level_node(max_height);
        nodes += txid_num;

        for _ in 0..nodes {
            tree._insert_hash([0; 32]);
        }

        tree._set_txid_nodes();
        tree
    }

    /// Returns the number of non empty node hashes the tree has
    fn _get_hashes_count(&self) -> usize {
        self.hashes_count
    }

    /// Returns the hashes of the block, in order
    fn _get_hashes(&self) -> Vec<[u8; 32]> {
        self.hashes.clone()
    }

    fn _get_first_level_node(max_height: usize) -> usize {
        let mut node = 0;
        for i in 0..max_height {
            node += 2usize.pow(i as u32);
        }
        node
    }

    /// Returns the max height for a number of txids
    fn _get_max_height(txid_num: usize) -> usize {
        if txid_num == 1 {
            return 1;
        }
        (f32::log2(txid_num as f32)).ceil() as usize
    }

    /// Searches for the tree's leafs and set its  `is_txid` attribute to `true`
    fn _set_txid_nodes(&mut self) {
        let mut root = match self.root.as_ref() {
            Some(root) => Rc::clone(root),
            None => return,
        };

        self._set_txid_nodes_recursive(&mut root);
    }

    /// Recursive auxiliar func to set the leaf's `is_txid` attribute to `true`
    fn _set_txid_nodes_recursive(&self, node: &mut Rc<RefCell<Box<MerkleNode>>>) {
        let max_height = Self::_calculate_height(self.count);

        if !node.as_ref().borrow().has_left()
            && !node.as_ref().borrow().has_right()
            && node.as_ref().borrow().height == max_height
        {
            node.as_ref().borrow_mut().is_txid = true;
            return;
        }

        let mut left = match node.as_ref().borrow().left().as_ref() {
            Some(left) => Rc::clone(left),
            None => return,
        };
        self._set_txid_nodes_recursive(&mut left);

        let mut right = match node.as_ref().borrow().right().as_ref() {
            Some(right) => Rc::clone(right),
            None => return,
        };
        self._set_txid_nodes_recursive(&mut right);
    }

    /// Returns the height of the tree based on how many nodes it has
    fn _calculate_height(elem_count: usize) -> usize {
        (f32::log2(elem_count as f32)).floor() as usize
    }

    // Inserts a new hash into the Merkle Tree
    fn _insert_hash(&mut self, hash: [u8; 32]) {
        let height = Self::_calculate_height(self.count + 1); // We sum 1 for the current node being inserted
        let new_node = Box::new(MerkleNode::new(hash, height, self.count));

        let root_is_some = self.root.is_some();
        if root_is_some {
            let root = match self.root.as_ref() {
                Some(root) => Rc::clone(root),
                None => return,
            };
            let mut queue: Queue<MerkleRoot> = Queue::new();
            queue.enqueue(Rc::clone(&root));
            self._insert_node(&mut queue, new_node);
        } else {
            self.root = Some(Rc::new(RefCell::new(new_node)));
        }

        self.count += 1;
    }

    /// Helper function: inserts a new node in level order
    fn _insert_node(&mut self, queue: &mut Queue<MerkleRoot>, new_node: Box<MerkleNode>) {
        while queue.size() > 0 {
            let parent = match queue.dequeue() {
                Some(parent) => parent,
                None => return,
            };

            let has_left = parent.as_ref().borrow().has_left();
            let has_right = parent.as_ref().borrow().has_right();

            // If it doesn't have a left child, it inserts the new node in that position
            if !has_left {
                parent
                    .as_ref()
                    .borrow_mut()
                    .save_left(Some(Rc::new(RefCell::new(new_node))));
                return;
            } else {
                queue.enqueue(Rc::clone(parent.as_ref().borrow().left().as_ref().unwrap()));
            }

            // If it doesn't have a right child, it inserts the new node in that position
            if !has_right {
                parent
                    .as_ref()
                    .borrow_mut()
                    .save_right(Some(Rc::new(RefCell::new(new_node))));
                return;
            } else {
                queue.enqueue(Rc::clone(
                    parent.as_ref().borrow().right().as_ref().unwrap(),
                ));
            }
        }
    }

    /// Maps a Merkle Block Message, with its flags and hashes, into the tree.
    /// It is recommended for the tree to be empty
    pub fn map_merkle_block_msg(&self, flags: &[u8], hashes: Vec<[u8; 32]>) {
        let mut queue: Queue<[u8; 32]> = Queue::new();
        for h in hashes {
            queue.enqueue(h);
        }

        if let Some(root) = self.root.as_ref() {
            self._map_merkle_block_msg_recursive(root, flags, 0, &mut queue);
        }
    }

    /// Auxiliar function for mapping a Merkle Block Message into the tree
    fn _map_merkle_block_msg_recursive(
        &self,
        node: &MerkleRoot,
        flags: &[u8],
        index: usize,
        hashes_queue: &mut Queue<[u8; 32]>,
    ) -> usize {
        let mut node = node.as_ref().borrow_mut();
        let mut flag_index = index;

        if flag_index >= flags.len() {
            return flag_index;
        }

        let current_hash = match hashes_queue.dequeue() {
            Some(hash) => hash,
            None => return flag_index,
        };
        println!("\ncurrent_hash: {:?}", current_hash);
        // Self::_print_node(node.as_ref());

        if node.is_txid && flags[flag_index] == 0 {
            // node is txid and flag = 0
            self._zero_txid(&mut node, current_hash);
        } else if node.is_txid && flags[flag_index] == 1 {
            // node is txid and flag = 1
            self._one_txid(&mut node, current_hash);
        } else if flags[flag_index] == 0 {
            // node not txid and flag = 0
            self._zero_non_txid(&mut node, current_hash);
        } else {
            // node not txid and flag = 1
            hashes_queue.prepend(current_hash); // the hash is not going to be used
            self._one_non_txid(&mut node);
        }

        let mut childs: Vec<&OptionalMerkleRoot> = vec![];
        if node.process_descendants && node.left().as_ref().is_some() {
            childs.push(node.left());
        }
        if node.process_descendants && node.right().as_ref().is_some() {
            childs.push(node.right());
        }

        for c in childs.iter() {
            let child = match c {
                Some(child) => Rc::clone(child),
                None => return flag_index,
            };
            flag_index =
                self._map_merkle_block_msg_recursive(&child, flags, flag_index + 1, hashes_queue);
        }

        flag_index
    }

    /// Function to call when flag is zero and node is txid.
    /// Saves the current hash in the node
    fn _zero_txid(&self, node: &mut MerkleNode, hash: [u8; 32]) {
        println!("zero txid");
        node.borrow_mut().hash = hash;
    }

    /// Function to call when flag is one and node is txid
    /// Saves the current hash in the node and marks it as matched
    fn _one_txid(&self, node: &mut MerkleNode, hash: [u8; 32]) {
        println!("one txid");
        node.borrow_mut().hash = hash;
        node.borrow_mut().matched = true;
    }

    /// Function to call when flag is zero and node is not txid
    /// Saves the current hash in the node and marks it as not needing to process its descendants
    fn _zero_non_txid(&self, node: &mut MerkleNode, hash: [u8; 32]) {
        println!("zero non-txid");
        node.borrow_mut().hash = hash;
        node.borrow_mut().process_descendants = false;
    }

    /// Function to call when flag is one and node is not txid
    /// Leaves the hash empty and marks it as needed to be computed from its children
    fn _one_non_txid(&self, node: &mut MerkleNode) {
        println!("one non-txid");
        node.borrow_mut().needs_computing = true;
    }

    /// Calculates the empty hashes in the tree that are marked as `needs_computing`, in level order.
    fn _complete_tree_hashes(&self) {
        let root = self.root.clone();
        if root.is_none() {
            return;
        }
        let mut stack = Self::_get_level_order_nodes(&root);

        while let Some(current) = stack.pop() {
            let mut node = current.as_ref().borrow_mut();

            // Self::_print_node(&node);

            if (!node.has_left() && !node.has_right()) || !node.process_descendants {
                println!("no children or no process descendants");
                continue;
            }

            if node.hash == [0; 32] || node.needs_computing {
                let left_root: MerkleRoot;
                let right_root: MerkleRoot;

                // If it has left node, use it as left_root
                if let Some(left) = node.left().clone() {
                    // Unless the node has hash 0. Then it moves on to the next node
                    if left.as_ref().borrow().hash == [0; 32] {
                        continue;
                    };

                    left_root = left;

                    // If it has right node, use it as right_root
                    if let Some(right) = node.right().clone() {
                        // Unless the node has hash 0. Then it uses the left node as right_root too
                        if right.as_ref().borrow().hash == [0; 32] {
                            right_root = match node.left().clone() {
                                Some(left) => left,
                                None => return,
                            };
                        } else {
                            right_root = right;
                        }
                    } else {
                        // if it does not have a right node, use left_root as right_root too
                        right_root = match node.left().clone() {
                            Some(left) => left,
                            None => return,
                        };
                    }

                    node.hash = match Self::get_hash(&left_root, &right_root) {
                        Some(hash) => hash,
                        None => return,
                    };
                }
                // If it does not have any child nodes, the coinbase TXID is used as the merkle root hash, so it does not need a hash calculation
            }
        }
    }

    /// Returns the hash of the concatenation of the two hashes passed as parameters.
    fn get_hash(left: &MerkleRoot, right: &MerkleRoot) -> Option<[u8; 32]> {
        let serialized_left = left.as_ref().borrow().hash;
        let serialized_right = right.as_ref().borrow().hash;
        let serialized = [&serialized_left[..], &serialized_right[..]].concat();

        vec_calculate_doublehash_array_be(serialized)
    }

    pub fn proof_of_inclusion(block: &BlockMessage, tx: &TXMessage) -> bool {
        if block.tx_count.number == 1 {
            let txid = tx.get_id();
            return txid == reverse_array(&block.block_header.merkle_root_hash);
        }

        let (mut merkle_proof, mut is_left) = Self::proof_of_inclusion_tx(block, tx);
        if merkle_proof.is_empty() {
            return false;
        }
        let mut h1 = merkle_proof.remove(0);
        _ = is_left.remove(0);
        while !merkle_proof.is_empty() {
            let h2 = merkle_proof.remove(0);
            let h2_is_left = is_left.remove(0);

            let h1h2 = if !h2_is_left {
                [&h1[..], &h2[..]].concat()
            } else {
                [&h2[..], &h1[..]].concat()
            };

            h1 = match vec_calculate_doublehash_array_be(h1h2) {
                Some(h) => h,
                None => {
                    return false;
                }
            };
        }
        reverse_array(&h1) == block.block_header.merkle_root_hash
    }

    /// Returns true if the merkle block is valid, false otherwise.
    fn proof_of_inclusion_is_valid(
        tx_count: usize,
        hashes: Vec<[u8; 32]>,
        root_hash: [u8; 32],
        flags: &[u8],
    ) -> bool {
        let tree = Self::build_merkle_tree(tx_count, hashes, flags);
        let tree_root_hash = reverse_array(&tree.root.as_ref().unwrap().borrow().hash);

        println!("root hash: {:?}", root_hash);
        println!("tree root hash: {:?}", tree_root_hash);

        tree_root_hash == root_hash
    }

    fn build_merkle_tree(tx_count: usize, hashes: Vec<[u8; 32]>, flags: &[u8]) -> MerkleTree {
        let mut tree = MerkleTree::build_empty_tree(tx_count);

        tree.hashes = hashes.clone();
        tree.map_merkle_block_msg(flags, hashes);
        // Self::_print_level_order(&tree.root);

        tree.hashes_count = tree.hashes.len();

        // We compute the hashes above the transactions
        tree._complete_tree_hashes();

        tree
    }

    pub fn merkle_block_is_valid(merkle_block: &MerkleBlock) -> bool {
        let tx_count = merkle_block.transaction_count as usize;
        let hashes = merkle_block.hashes.clone();
        let root_hash = merkle_block.block_header.merkle_root_hash;
        let flags = cast_vec_to_binary_vec(merkle_block.flags.clone());

        if tx_count == 1 {
            return hashes[0] == reverse_array(&root_hash);
        }

        Self::proof_of_inclusion_is_valid(tx_count, hashes, root_hash, &flags)
    }

    pub fn get_built_merkle_tree(
        block: &BlockMessage,
        tx: &TXMessage,
    ) -> (MerkleTree, Option<usize>, Vec<u8>) {
        let tx_count = block.tx_count.get_number();
        let mut hashes: Vec<[u8; 32]> = Vec::new();
        let mut tx_pos = None;
        for (pos, t) in block.transaction_history.iter().enumerate() {
            // saves each txid in a hashes vec to insert later in the tree
            let h = t.get_id();
            // looks for the tx given as parameter
            if tx.serialize() == t.serialize() {
                // if it is found, it saves its position in the block
                println!("\nPOSITION FOUND IN BLOCK FOR TX:\n {:?}", pos);
                tx_pos = Some(pos);
            }
            hashes.push(h);
        }

        let max_height = Self::_get_max_height(tx_count);
        let nodes_count = Self::_get_first_level_node(max_height + 1);
        let flags = vec![1; nodes_count];

        (
            Self::build_merkle_tree(tx_count, hashes, &flags),
            tx_pos,
            flags,
        )
    }

    /// Returns true if the merkle root hash of the block is equal to the calculated merkle root hash from the block's txs merkle tree.
    pub fn proof_of_inclusion_tx(
        block: &BlockMessage,
        tx: &TXMessage,
    ) -> (Vec<[u8; 32]>, Vec<bool>) {
        let tx_count = block.tx_count.get_number();
        let (tree, tx_pos, _) = Self::get_built_merkle_tree(block, tx);

        let tx_pos_found = match tx_pos {
            Some(pos) => pos,
            None => {
                println!("POSITION NOT FOUND IN BLOCK FOR TX");
                return (Vec::new(), Vec::new());
            }
        };
        let (hashes, is_left) = tree.get_merkle_root_path(tx_pos_found, tx_count);
        (hashes, is_left)
    }

    /// Returns the merkle root path of a tx in a block.
    fn get_merkle_root_path(
        &self,
        tx_pos: usize,
        tx_count: usize,
    ) -> (Vec<[u8; 32]>, Vec<bool>) {
        let tx_pos_in_tree = self.get_tx_pos_in_tree(tx_pos, tx_count);
        let nodes = Self::_get_level_order_nodes(&self.root);
        let (hashes_positions_to_retreive, is_left): (Vec<usize>, Vec<bool>) =
            Self::_get_hashes_positions_to_retreive(tx_pos_in_tree);

        println!("merklefix hashes_positions_to_retreive: {:?}", hashes_positions_to_retreive);
        let mut hashes: Vec<[u8; 32]> = Vec::new();
        let max_hashes = nodes.len() - 1;
        
        for pos in hashes_positions_to_retreive {

            let mut node = if pos <= max_hashes {
                nodes[pos].clone()
            } else {
                nodes[max_hashes].clone()
            };

            node = if node.borrow().hash == [0; 32] {
                if node.borrow().is_right() {
                    nodes[pos - 1].clone()
                } else {
                    nodes[pos + 1].clone()
                }
            } else {
                node
            };

            let hash = node.borrow().hash;
            hashes.push(hash);
        }

        (hashes, is_left)
    }

    fn get_tx_pos_in_tree(&self, tx_number: usize, tx_count: usize) -> usize {
        let mut n = 0;
        let max_height = Self::_get_max_height(tx_count);
        for _ in 0..max_height {
            n = 2 * n + 1;
        }
        n + tx_number
    }

    fn _get_hashes_positions_to_retreive(tx_pos_in_tree: usize) -> (Vec<usize>, Vec<bool>) {
        let mut positions: Vec<usize> = Vec::new();
        let mut is_left = Vec::new();
        let mut n = tx_pos_in_tree;
        positions.push(n);
        is_left.push(n % 2 != 0);
        while n >= 1 {
            if n % 2 == 0 {
                // if is right child
                is_left.push(true);
                positions.push(n - 1);
            } else {
                // if is left child
                is_left.push(false);
                positions.push(n + 1);
            }
            n = (n - 1) / 2;
        }
        positions.sort(); // sorts the positions in ascending order
        positions.reverse(); // reverses the order of the positions so nodes are retrieved from bottom to top
        positions.swap(0, 1); // swaps the first two positions so the left node is always the first element of the vector for hashing
        if tx_pos_in_tree % 2 == 0 {
            // if tx_pos_in_tree is even, the tx is a right child. So we need to swap the positions
            is_left.swap(0, 1);
        }

        (positions, is_left)
    }

    /// Retrieves the number of stored nodes
    fn _get_hash_count(&self) -> usize {
        self.count
    }

    /// Prints the tree recursively in a pre-order way
    fn _print_recursive_pre_order(root: &Option<Rc<RefCell<Box<MerkleNode>>>>) {
        println!("\n** PRE-ORDER **");
        if root.is_none() {
            return;
        }

        if let Some(root_s) = root {
            let root = root_s.as_ref().borrow();
            Self::_print_node(&root);

            Self::_print_recursive_pre_order(root.left());
            Self::_print_recursive_pre_order(root.right());
        }
    }

    fn _get_level_order_nodes(
        root: &Option<Rc<RefCell<Box<MerkleNode>>>>,
    ) -> Vec<Rc<RefCell<Box<MerkleNode>>>> {
        let root = match root.as_ref() {
            Some(root) => Rc::clone(root),
            None => return Vec::new(),
        };

        let mut node_list: Vec<Rc<RefCell<Box<MerkleNode>>>> = Vec::new();

        let mut queue: Queue<Rc<RefCell<Box<MerkleNode>>>> = Queue::new();
        queue.enqueue(root);

        while queue.size() > 0 {
            let current = match queue.dequeue() {
                Some(current) => current,
                None => return node_list,
            };

            node_list.push(Rc::clone(&current));

            if current.as_ref().borrow().has_left() {
                let left = match current.as_ref().borrow().left() {
                    Some(left) => Rc::clone(left),
                    None => return node_list,
                };
                queue.enqueue(Rc::clone(&left));
            }

            if current.as_ref().borrow().has_right() {
                let right = match current.as_ref().borrow().right() {
                    Some(right) => Rc::clone(right),
                    None => return node_list,
                };
                queue.enqueue(Rc::clone(&right));
            }
        }
        node_list
    }

    fn _print_level_order(root: &Option<Rc<RefCell<Box<MerkleNode>>>>) {
        println!("\n** LEVEL ORDER **");
        let node_list = Self::_get_level_order_nodes(root);

        for n in node_list.iter() {
            Self::_print_node(&n.as_ref().borrow());
        }
    }

    fn _print_node(node: &MerkleNode) {
        println!(
            "\nNODE DETAIL\n------------\nposition {:?}, height {:?}, is_tixd: {:?}, process_desc: {:?}, matched: {:?}, needs_computing: {:?}, has_left: {:?}, has_right: {:?}, hash: [{:?}]",
            node.position, node.height, node.is_txid, node.process_descendants, node.matched, node.needs_computing, node.has_left(), node.has_right(), u8_array_to_hex_string(&node.hash)
        );
    }
}

#[cfg(test)]
mod merkle_tree_test {

    use crate::message_structs::{
        block_headers::BlockHeader, compact_size::CompactSize, input::Input, outpoint::Outpoint,
        output::Output,
    };

    use super::*;

    #[test]
    fn test_calculate_height() {
        let height = MerkleTree::_calculate_height(1);
        assert_eq!(height, 0);

        let height = MerkleTree::_calculate_height(2);
        assert_eq!(height, 1);

        let height = MerkleTree::_calculate_height(3);
        assert_eq!(height, 1);

        let height = MerkleTree::_calculate_height(4);
        assert_eq!(height, 2);

        let height = MerkleTree::_calculate_height(6);
        assert_eq!(height, 2);

        let height = MerkleTree::_calculate_height(14);
        assert_eq!(height, 3);
    }

    #[test]
    fn test_build_empty_tree_counts() {
        // Initialize a new Merkle Tree
        let number = 1;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 2);

        let number = 2;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 3);

        let number = 3;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 6);

        let number = 4;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 7);

        let number = 5;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 12);

        let number = 6;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 13);

        let number = 7;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 14);

        let number = 8;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 15);

        let number = 9;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 24);

        let number = 10;
        let merkle_tree = MerkleTree::build_empty_tree(number);
        assert_eq!(merkle_tree.count, 25);
    }

    #[test]
    fn test_merkle_tree_hash_count_ok() {
        // Initialize a new Merkle Tree
        let mut merkle_tree = MerkleTree::new();

        // Insert hashes into the Merkle Tree
        let hash1: [u8; 32] = [1u8; 32];
        let hash2: [u8; 32] = [2u8; 32];
        let hash3: [u8; 32] = [3u8; 32];

        let hashes = vec![hash1, hash2, hash3];

        for h in &hashes {
            merkle_tree._insert_hash(*h);
        }

        let hash_count = merkle_tree._get_hash_count();
        assert_eq!(hash_count, hashes.len())
    }

    #[test]
    fn test_set_txid_nodes() {
        // Initialize a new Merkle Tree
        let mut merkle_tree = MerkleTree::new();

        // Insert hashes into the Merkle Tree
        let hash1: [u8; 32] = [1u8; 32];
        let hash2: [u8; 32] = [2u8; 32];
        let hash3: [u8; 32] = [3u8; 32];

        let hashes = vec![hash1, hash2, hash3];

        for h in &hashes {
            merkle_tree._insert_hash(*h);
        }

        merkle_tree._set_txid_nodes();

        let root = merkle_tree.root.clone().unwrap();
        let root = root.borrow();
        assert!(!root.is_txid);
        assert!(root.left().as_ref().unwrap().borrow().is_txid);
        assert!(root.right().as_ref().unwrap().borrow().is_txid);
    }

    #[test]
    fn test_calculate_merkle_tree_two_txids() {
        let merkle_root_hash = [
            123, 147, 70, 237, 206, 22, 166, 209, 92, 148, 87, 159, 54, 214, 242, 45, 180, 54, 188,
            169, 11, 150, 135, 163, 250, 188, 31, 39, 136, 154, 79, 14,
        ];
        // c19a0c9424287a71b6ef82ae70d55036a593e3a3f2fdb90732b666d2e8496f51
        println!(
            "merkle root hash: {}",
            u8_array_to_hex_string(&merkle_root_hash)
        );

        let merkle_block = MerkleBlock::new(
            BlockHeader {
                version: 551550976,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 25, 68, 23, 170, 151, 163, 125, 21, 54, 5, 235, 29, 131,
                    225, 122, 8, 252, 39, 122, 32, 29, 162, 220, 186, 92,
                ],
                merkle_root_hash,
                time: 1687743341,
                n_bits: 421617023,
                nonce: 3183852326,
            },
            2,
            CompactSize::from_usize_to_compact_size(2),
            vec![
                [
                    92, 145, 253, 116, 227, 31, 2, 208, 177, 80, 62, 255, 37, 64, 166, 226, 168,
                    145, 6, 219, 143, 126, 51, 21, 108, 172, 148, 120, 235, 219, 44, 216,
                ],
                [
                    237, 159, 232, 21, 104, 101, 95, 37, 203, 48, 4, 162, 179, 236, 93, 231, 7, 75,
                    234, 151, 241, 232, 240, 140, 170, 0, 25, 137, 106, 186, 94, 191,
                ],
            ], // hashes
            CompactSize {
                prefix: 0,
                number_vec: vec![1],
                number: 1,
            },
            vec![7],
        );

        let valid = MerkleTree::merkle_block_is_valid(&merkle_block);

        assert!(valid)
    }

    #[test]
    fn test_calculate_merkle_tree_three_txid() {
        let merkle_root_hash = [
            68, 105, 32, 210, 163, 144, 18, 139, 171, 201, 106, 56, 115, 178, 1, 62, 64, 220, 108,
            212, 193, 176, 26, 145, 116, 7, 124, 27, 181, 109, 13, 76,
        ];

        let merkle_block = MerkleBlock::new(
            BlockHeader {
                version: 538968064,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 6, 100, 133, 196, 245, 88, 200, 116, 150, 15, 103, 175,
                    99, 87, 172, 34, 23, 170, 222, 172, 11, 157, 179, 74, 91,
                ],
                merkle_root_hash,
                time: 1687959440,
                n_bits: 421842426,
                nonce: 2844730702,
            },
            3,
            CompactSize::from_usize_to_compact_size(3),
            vec![
                [
                    126, 234, 72, 147, 157, 11, 72, 60, 100, 218, 153, 53, 245, 171, 100, 150, 182,
                    197, 136, 61, 155, 247, 188, 41, 140, 26, 238, 234, 72, 98, 172, 129,
                ],
                [
                    118, 185, 236, 150, 140, 91, 117, 133, 23, 60, 63, 198, 44, 70, 49, 221, 86,
                    12, 63, 12, 93, 241, 89, 94, 91, 4, 33, 222, 226, 148, 97, 13,
                ],
                [
                    138, 166, 243, 19, 88, 94, 9, 204, 183, 46, 150, 127, 224, 73, 29, 212, 222,
                    55, 40, 139, 113, 209, 11, 180, 116, 229, 82, 185, 129, 110, 137, 148,
                ],
            ],
            CompactSize::from_usize_to_compact_size(1),
            vec![63],
        );

        let valid = MerkleTree::merkle_block_is_valid(&merkle_block);

        assert!(valid)
    }

    #[test]
    fn test_calculate_merkle_tree_ten_txid() {
        let merkle_block = MerkleBlock::new(
            BlockHeader {
                version: 626909184,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 22, 28, 212, 70, 68, 72, 222, 132, 236, 25, 121, 52, 142,
                    217, 83, 89, 252, 169, 193, 175, 252, 236, 212, 136, 134,
                ],
                merkle_root_hash: [
                    133, 201, 108, 140, 174, 145, 167, 229, 35, 166, 159, 109, 169, 152, 8, 212,
                    166, 122, 94, 127, 244, 25, 145, 144, 206, 27, 115, 230, 4, 8, 143, 82,
                ],
                time: 1687965379,
                n_bits: 421842426,
                nonce: 610490495,
            },
            10,
            CompactSize::from_usize_to_compact_size(10),
            vec![
                [
                    197, 33, 133, 83, 29, 30, 90, 182, 73, 167, 80, 62, 71, 35, 222, 55, 170, 3,
                    26, 114, 73, 86, 216, 254, 78, 38, 162, 250, 16, 141, 126, 241,
                ],
                [
                    20, 178, 212, 143, 170, 80, 158, 216, 41, 106, 137, 170, 7, 72, 207, 73, 51,
                    31, 103, 77, 81, 99, 39, 148, 17, 19, 130, 193, 176, 128, 121, 56,
                ],
                [
                    60, 146, 43, 132, 113, 217, 68, 189, 112, 171, 193, 29, 142, 55, 83, 115, 141,
                    189, 155, 30, 193, 67, 200, 243, 83, 42, 166, 145, 201, 250, 127, 11,
                ],
                [
                    214, 158, 155, 112, 195, 240, 206, 136, 45, 90, 71, 41, 9, 236, 181, 58, 94,
                    186, 213, 203, 183, 143, 174, 147, 0, 161, 178, 190, 122, 248, 224, 3,
                ],
                [
                    239, 236, 150, 37, 152, 161, 108, 242, 28, 56, 152, 27, 228, 28, 34, 6, 18,
                    184, 159, 186, 43, 165, 80, 183, 4, 146, 199, 40, 221, 27, 112, 175,
                ],
                [
                    110, 128, 218, 188, 219, 62, 131, 97, 141, 26, 231, 96, 140, 214, 83, 112, 103,
                    219, 6, 251, 128, 241, 56, 85, 186, 234, 232, 87, 50, 72, 89, 215,
                ],
                [
                    57, 229, 117, 63, 174, 198, 230, 138, 254, 56, 229, 165, 80, 35, 178, 45, 159,
                    165, 182, 54, 80, 48, 43, 206, 21, 74, 8, 111, 86, 167, 24, 195,
                ],
                [
                    121, 114, 207, 70, 20, 32, 205, 0, 125, 89, 147, 125, 70, 120, 156, 17, 253,
                    52, 120, 28, 49, 149, 8, 178, 4, 45, 244, 60, 100, 112, 76, 252,
                ],
                [
                    209, 178, 21, 235, 48, 203, 75, 180, 88, 115, 158, 12, 5, 208, 195, 130, 197,
                    74, 80, 250, 152, 111, 149, 71, 28, 207, 81, 72, 77, 221, 26, 68,
                ],
                [
                    195, 251, 178, 76, 13, 63, 191, 146, 156, 193, 63, 58, 202, 122, 38, 107, 222,
                    239, 239, 174, 10, 91, 131, 160, 214, 200, 58, 86, 145, 217, 31, 82,
                ],
            ],
            CompactSize::from_usize_to_compact_size(3),
            vec![255, 255, 31],
        );

        let valid = MerkleTree::merkle_block_is_valid(&merkle_block);

        assert!(valid)
    }

    #[test]
    fn test_calculate_merkle_tree_is_valid_single_txid() {
        let merkle_block = MerkleBlock::new(
            BlockHeader {
                version: 536870916,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 4, 172, 233, 200, 163, 189, 77, 242, 128, 236, 3, 8, 37,
                    159, 50, 16, 171, 52, 169, 186, 53, 79, 174, 31, 129,
                ],
                merkle_root_hash: [
                    116, 251, 168, 167, 69, 91, 101, 165, 125, 196, 223, 168, 77, 74, 253, 145,
                    131, 234, 157, 136, 111, 230, 60, 53, 169, 190, 244, 230, 171, 90, 140, 34,
                ],
                time: 1687982238,
                n_bits: 421842426,
                nonce: 3458162242,
            },
            1,
            CompactSize::from_usize_to_compact_size(1),
            vec![[
                34, 140, 90, 171, 230, 244, 190, 169, 53, 60, 230, 111, 136, 157, 234, 131, 145,
                253, 74, 77, 168, 223, 196, 125, 165, 101, 91, 69, 167, 168, 251, 116,
            ]],
            CompactSize::from_usize_to_compact_size(1),
            vec![1],
        );

        let valid = MerkleTree::merkle_block_is_valid(&merkle_block);

        assert!(valid)
    }

    #[test]
    fn test_get_max_height() {
        assert_eq!(MerkleTree::_get_max_height(1), 1);
        assert_eq!(MerkleTree::_get_max_height(2), 1);
        assert_eq!(MerkleTree::_get_max_height(3), 2);
        assert_eq!(MerkleTree::_get_max_height(4), 2);
        assert_eq!(MerkleTree::_get_max_height(5), 3);
        assert_eq!(MerkleTree::_get_max_height(6), 3);
        assert_eq!(MerkleTree::_get_max_height(9), 4);
        assert_eq!(MerkleTree::_get_max_height(14), 4);
    }

    #[test]
    fn test_proof_of_inclusion() {
        let block = get_block_message();
        println!("{:?}", block);
        for tx in block.transaction_history.iter() {
            assert!(MerkleTree::proof_of_inclusion(&block, tx));
        }
    }

    #[test]
    fn test_proof_of_inclusion_single_tx() {
        let block = get_block_single_tx();
        println!("{:?}", block);
        let tx = block.transaction_history[0].clone();
        assert!(MerkleTree::proof_of_inclusion(&block, &tx));
    }

    fn get_block_message() -> BlockMessage {
        BlockMessage {
            block_header: BlockHeader {
                version: 672776192,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 12, 146, 168, 148, 210, 10, 126, 9, 121, 161, 49, 252,
                    115, 220, 179, 60, 220, 113, 84, 188, 49, 24, 110, 122, 176,
                ],
                merkle_root_hash: [
                    239, 118, 126, 120, 206, 108, 123, 34, 247, 104, 39, 76, 53, 132, 53, 156, 214,
                    227, 4, 154, 130, 39, 10, 144, 138, 223, 245, 93, 15, 38, 191, 1,
                ],
                time: 1688342584,
                n_bits: 421842426,
                nonce: 3933657369,
            },
            tx_count: CompactSize {
                prefix: 0,
                number_vec: vec![5],
                number: 5,
            },
            transaction_history: vec![
                TXMessage::new(
                    1,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![1],
                        number: 1,
                    },
                    vec![Input::new(
                        Outpoint::new(
                            [
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            ],
                            4294967295,
                        ),
                        CompactSize {
                            prefix: 0,
                            number_vec: vec![30],
                            number: 30,
                        },
                        vec![
                            3, 169, 59, 37, 25, 68, 77, 71, 66, 108, 111, 99, 107, 99, 104, 97,
                            105, 110, 55, 16, 77, 144, 64, 0, 0, 0, 0, 0, 0, 0,
                        ],
                        4294967295,
                    )],
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![2],
                        number: 2,
                    },
                    vec![
                        Output {
                            value: 2449071,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![22],
                                number: 22,
                            },
                            script: vec![
                                0, 20, 178, 55, 22, 225, 131, 186, 9, 73, 197, 93, 108, 172, 33,
                                163, 233, 65, 118, 238, 209, 18,
                            ],
                        },
                        Output {
                            value: 0,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![38],
                                number: 38,
                            },
                            script: vec![
                                106, 36, 170, 33, 169, 237, 38, 26, 12, 78, 105, 109, 211, 25, 45,
                                36, 136, 94, 199, 105, 235, 28, 37, 85, 245, 232, 174, 189, 155,
                                67, 163, 214, 36, 52, 61, 181, 75, 116,
                            ],
                        },
                    ],
                    0,
                ),
                TXMessage::new(
                    1,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![1],
                        number: 1,
                    },
                    vec![Input::new(
                        Outpoint::new(
                            [
                                52, 30, 81, 110, 241, 33, 151, 13, 107, 183, 203, 234, 138, 150,
                                150, 72, 173, 197, 222, 160, 114, 97, 43, 136, 179, 215, 240, 138,
                                226, 218, 157, 49,
                            ],
                            3,
                        ),
                        CompactSize {
                            prefix: 0,
                            number_vec: vec![106],
                            number: 106,
                        },
                        vec![
                            71, 48, 68, 2, 32, 36, 39, 218, 209, 197, 54, 159, 44, 196, 225, 47,
                            215, 129, 148, 251, 56, 17, 222, 199, 157, 215, 144, 202, 249, 204,
                            205, 230, 194, 253, 132, 159, 178, 2, 32, 5, 198, 23, 218, 202, 170,
                            122, 146, 91, 186, 187, 201, 36, 100, 110, 92, 9, 56, 247, 169, 24, 31,
                            69, 8, 181, 79, 53, 166, 187, 71, 44, 33, 1, 33, 3, 116, 53, 193, 148,
                            233, 176, 27, 61, 127, 122, 40, 2, 214, 104, 74, 58, 246, 141, 5, 187,
                            244, 236, 143, 23, 2, 25, 128, 215, 119, 105, 31, 29,
                        ],
                        4294967293,
                    )],
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![4],
                        number: 4,
                    },
                    vec![
                        Output {
                            value: 0,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![83],
                                number: 83,
                            },
                            script: vec![
                                106, 76, 80, 84, 50, 91, 82, 163, 164, 76, 12, 46, 112, 221, 205,
                                44, 21, 146, 252, 1, 122, 148, 129, 217, 16, 193, 30, 87, 96, 2,
                                234, 239, 255, 76, 180, 22, 183, 120, 149, 105, 107, 34, 138, 32,
                                8, 220, 184, 109, 215, 229, 68, 144, 113, 108, 100, 23, 31, 136,
                                187, 207, 7, 182, 173, 240, 230, 74, 224, 174, 169, 216, 0, 37, 59,
                                168, 0, 3, 0, 37, 57, 175, 0, 11, 76,
                            ],
                        },
                        Output {
                            value: 10000,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![25],
                                number: 25,
                            },
                            script: vec![
                                118, 169, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                0, 0, 136, 172,
                            ],
                        },
                        Output {
                            value: 10000,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![25],
                                number: 25,
                            },
                            script: vec![
                                118, 169, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                0, 0, 136, 172,
                            ],
                        },
                        Output {
                            value: 365957408,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![25],
                                number: 25,
                            },
                            script: vec![
                                118, 169, 20, 186, 39, 249, 158, 0, 124, 127, 96, 90, 131, 5, 227,
                                24, 193, 171, 222, 60, 210, 32, 172, 136, 172,
                            ],
                        },
                    ],
                    0,
                ),
                TXMessage::new(
                    1,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![1],
                        number: 1,
                    },
                    vec![Input::new(
                        Outpoint::new(
                            [
                                49, 157, 128, 32, 200, 93, 163, 172, 26, 109, 226, 66, 74, 72, 16,
                                82, 147, 233, 65, 39, 183, 224, 155, 89, 33, 29, 25, 125, 107, 129,
                                127, 252,
                            ],
                            1,
                        ),
                        CompactSize {
                            prefix: 0,
                            number_vec: vec![0],
                            number: 0,
                        },
                        vec![],
                        4294967295,
                    )],
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![2],
                        number: 2,
                    },
                    vec![
                        Output {
                            value: 0,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![22],
                                number: 22,
                            },
                            script: vec![
                                0, 20, 199, 131, 6, 139, 37, 147, 199, 19, 141, 135, 68, 149, 111,
                                157, 4, 128, 50, 197, 128, 128,
                            ],
                        },
                        Output {
                            value: 255297383,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![22],
                                number: 22,
                            },
                            script: vec![
                                0, 20, 199, 131, 6, 139, 37, 147, 199, 19, 141, 135, 68, 149, 111,
                                157, 4, 128, 50, 197, 128, 128,
                            ],
                        },
                    ],
                    0,
                ),
                TXMessage::new(
                    1,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![1],
                        number: 1,
                    },
                    vec![Input::new(
                        Outpoint::new(
                            [
                                201, 16, 58, 50, 2, 48, 169, 91, 197, 33, 141, 147, 130, 100, 142,
                                170, 242, 224, 72, 167, 236, 58, 241, 222, 130, 3, 183, 196, 177,
                                179, 243, 98,
                            ],
                            1,
                        ),
                        CompactSize {
                            prefix: 0,
                            number_vec: vec![0],
                            number: 0,
                        },
                        vec![],
                        4294967295,
                    )],
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![2],
                        number: 2,
                    },
                    vec![
                        Output {
                            value: 121,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![25],
                                number: 25,
                            },
                            script: vec![
                                118, 169, 20, 156, 75, 18, 187, 90, 46, 126, 75, 39, 33, 162, 93,
                                138, 190, 189, 106, 129, 68, 212, 18, 136, 172,
                            ],
                        },
                        Output {
                            value: 255296968,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![22],
                                number: 22,
                            },
                            script: vec![
                                0, 20, 199, 131, 6, 139, 37, 147, 199, 19, 141, 135, 68, 149, 111,
                                157, 4, 128, 50, 197, 128, 128,
                            ],
                        },
                    ],
                    0,
                ),
                TXMessage::new(
                    2,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![1],
                        number: 1,
                    },
                    vec![Input::new(
                        Outpoint::new(
                            [
                                139, 82, 119, 103, 18, 227, 78, 240, 228, 131, 215, 103, 244, 129,
                                49, 202, 223, 117, 236, 38, 129, 254, 42, 234, 23, 104, 83, 15,
                                160, 219, 140, 24,
                            ],
                            1,
                        ),
                        CompactSize {
                            prefix: 0,
                            number_vec: vec![23],
                            number: 23,
                        },
                        vec![
                            22, 0, 20, 66, 97, 134, 30, 247, 65, 146, 173, 11, 160, 201, 65, 9, 44,
                            60, 211, 82, 45, 229, 200,
                        ],
                        4294967293,
                    )],
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![3],
                        number: 3,
                    },
                    vec![
                        Output {
                            value: 60060,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![23],
                                number: 23,
                            },
                            script: vec![
                                169, 20, 146, 45, 87, 229, 136, 146, 92, 91, 190, 81, 99, 108, 94,
                                120, 145, 122, 220, 194, 237, 163, 135,
                            ],
                        },
                        Output {
                            value: 128836,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![23],
                                number: 23,
                            },
                            script: vec![
                                169, 20, 54, 164, 143, 133, 36, 64, 255, 14, 182, 164, 211, 209, 1,
                                115, 189, 11, 84, 226, 250, 187, 135,
                            ],
                        },
                        Output {
                            value: 91097,
                            script_length: CompactSize {
                                prefix: 0,
                                number_vec: vec![23],
                                number: 23,
                            },
                            script: vec![
                                169, 20, 117, 9, 184, 55, 205, 241, 101, 92, 19, 234, 99, 164, 92,
                                137, 146, 238, 4, 18, 122, 76, 135,
                            ],
                        },
                    ],
                    0,
                ),
            ],
        }
    }

    fn _get_tx_message() -> TXMessage {
        TXMessage::new(
            1,
            CompactSize {
                prefix: 0,
                number_vec: vec![1],
                number: 1,
            },
            vec![Input::new(
                Outpoint::new(
                    [
                        49, 157, 128, 32, 200, 93, 163, 172, 26, 109, 226, 66, 74, 72, 16, 82, 147,
                        233, 65, 39, 183, 224, 155, 89, 33, 29, 25, 125, 107, 129, 127, 252,
                    ],
                    1,
                ),
                CompactSize {
                    prefix: 0,
                    number_vec: vec![0],
                    number: 0,
                },
                vec![],
                4294967295,
            )],
            CompactSize {
                prefix: 0,
                number_vec: vec![2],
                number: 2,
            },
            vec![
                Output::new(
                    0,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![22],
                        number: 22,
                    },
                    vec![
                        0, 20, 199, 131, 6, 139, 37, 147, 199, 19, 141, 135, 68, 149, 111, 157, 4,
                        128, 50, 197, 128, 128,
                    ],
                ),
                Output::new(
                    255297383,
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![22],
                        number: 22,
                    },
                    vec![
                        0, 20, 199, 131, 6, 139, 37, 147, 199, 19, 141, 135, 68, 149, 111, 157, 4,
                        128, 50, 197, 128, 128,
                    ],
                ),
            ],
            0,
        )
    }

    fn get_block_single_tx() -> BlockMessage {
        BlockMessage {
            block_header: BlockHeader {
                version: 627851264,
                previous_block_header_hash: [
                    0, 0, 0, 0, 0, 0, 0, 7, 3, 13, 70, 140, 4, 6, 218, 138, 24, 242, 43, 154, 0,
                    40, 13, 229, 213, 97, 219, 35, 165, 174, 77, 217,
                ],
                merkle_root_hash: [
                    188, 169, 105, 48, 118, 96, 5, 68, 126, 12, 51, 147, 232, 36, 74, 126, 171,
                    121, 181, 100, 203, 215, 123, 6, 193, 147, 137, 38, 134, 174, 215, 227,
                ],
                time: 1687746781,
                n_bits: 421617023,
                nonce: 3360524581,
            },
            tx_count: CompactSize {
                prefix: 0,
                number_vec: vec![1],
                number: 1,
            },
            transaction_history: vec![TXMessage::new(
                1,
                CompactSize {
                    prefix: 0,
                    number_vec: vec![1],
                    number: 1,
                },
                vec![Input::new(
                    Outpoint::new(
                        [
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0,
                        ],
                        4294967295,
                    ),
                    CompactSize {
                        prefix: 0,
                        number_vec: vec![95],
                        number: 95,
                    },
                    vec![
                        3, 229, 55, 37, 4, 221, 248, 152, 100, 47, 77, 97, 114, 97, 83, 116, 97,
                        103, 101, 51, 47, 250, 190, 109, 109, 185, 99, 60, 107, 158, 171, 213, 74,
                        125, 198, 158, 51, 253, 137, 123, 193, 165, 36, 92, 168, 8, 87, 113, 43,
                        61, 62, 149, 186, 144, 152, 153, 149, 1, 0, 0, 0, 0, 0, 0, 0, 90, 199, 35,
                        247, 55, 84, 0, 101, 87, 5, 217, 81, 123, 207, 41, 40, 18, 247, 189, 189,
                        208, 0, 70, 0, 0, 98, 255, 255, 255, 255,
                    ],
                    4294967295,
                )],
                CompactSize {
                    prefix: 0,
                    number_vec: vec![2],
                    number: 2,
                },
                vec![
                    Output {
                        value: 2441406,
                        script_length: CompactSize {
                            prefix: 0,
                            number_vec: vec![25],
                            number: 25,
                        },
                        script: vec![
                            118, 169, 20, 227, 89, 246, 149, 200, 15, 201, 247, 25, 36, 70, 205,
                            201, 74, 175, 160, 7, 250, 226, 230, 136, 172,
                        ],
                    },
                    Output {
                        value: 0,
                        script_length: CompactSize {
                            prefix: 0,
                            number_vec: vec![38],
                            number: 38,
                        },
                        script: vec![
                            106, 36, 170, 33, 169, 237, 226, 246, 28, 63, 113, 209, 222, 253, 63,
                            169, 153, 223, 163, 105, 83, 117, 92, 105, 6, 137, 121, 153, 98, 180,
                            139, 235, 216, 54, 151, 78, 140, 249,
                        ],
                    },
                ],
                3290497474,
            )],
        }
    }
}
