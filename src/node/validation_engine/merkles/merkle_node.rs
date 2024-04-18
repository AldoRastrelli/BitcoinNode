use std::{cell::RefCell, rc::Rc};

pub type MerkleRoot = Rc<RefCell<Box<MerkleNode>>>;
pub type OptionalMerkleRoot = Option<MerkleRoot>;

#[derive(Debug)]
/// Data structure for the Merkle Tree node
pub struct MerkleNode {
    pub hash: [u8; 32],
    left: OptionalMerkleRoot,
    has_left: bool,
    right: OptionalMerkleRoot,
    has_right: bool,
    pub height: usize,
    pub position: usize,
    pub is_txid: bool,
    pub matched: bool,
    pub process_descendants: bool,
    pub needs_computing: bool,
}

impl Clone for MerkleNode {
    fn clone(&self) -> Self {
        MerkleNode {
            hash: self.hash,
            left: self.left.clone(),
            has_left: self.has_left,
            right: self.right.clone(),
            has_right: self.has_right,
            height: self.height,
            position: self.position,
            is_txid: self.is_txid,
            matched: self.matched,
            process_descendants: self.process_descendants,
            needs_computing: self.needs_computing,
        }
    }
}

impl MerkleNode {
    pub fn new(hash: [u8; 32], height: usize, position: usize) -> Self {
        MerkleNode {
            hash,
            left: None,
            has_left: false,
            right: None,
            has_right: false,
            height,
            position,
            is_txid: false,
            matched: false,
            process_descendants: true,
            needs_computing: false,
        }
    }

    pub fn left(&self) -> &OptionalMerkleRoot {
        &self.left
    }

    pub fn right(&self) -> &OptionalMerkleRoot {
        &self.right
    }

    pub fn save_left(&mut self, left: OptionalMerkleRoot) {
        self.left = left;
        self.has_left = true;
    }

    pub fn save_right(&mut self, right: OptionalMerkleRoot) {
        self.right = right;
        self.has_right = true;
    }

    pub fn has_left(&self) -> bool {
        self.has_left
    }

    pub fn has_right(&self) -> bool {
        self.has_right
    }

    pub fn is_left(&self) -> bool {
        self.position % 2 != 0
    }

    pub fn is_right(&self) -> bool {
        self.position % 2 == 0
    }
}
