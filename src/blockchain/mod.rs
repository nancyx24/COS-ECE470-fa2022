use crate::types::hash::H256;
use crate::types::hash::Hashable;
use std::collections::HashMap;
use super::types::block::{self, Block, Header, Content};
use super::types::transaction;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Blockchain {
    block_hash: HashMap<H256, Block>, // key = hash, value = block
    length_hash: HashMap<H256, u128>, // key = hash, value = length of block
    tip: H256, // last block's hash in longest chain
    longest_length: u128, // length of longest chain
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // MY CODE

        // create genesis block
        let nonce: u32 = 0;
        let zeros: [u8; 32] = [0; 32];
        let parent: H256 = H256::from(zeros);
        let difficulty: H256 = [100u8;32].into();
        let timestamp = 0;
        let content_data: Vec<transaction::SignedTransaction> = Vec::new();
        let data: Content = block::build_content(content_data);

        // merkle root of empty input
        // FOR NOW, BUT NEED TO IMPLEMENT IN MERKLE.RS
        let merkle_root: H256 = H256::from(zeros); 
        
        let header: Header = block::build_header(parent, nonce, difficulty, timestamp, merkle_root);
        let genesis: Block = block::build_block(header, data);
        let genesis_hash = genesis.hash();

        // create contents for Blockchain
        let mut block_hash = HashMap::new();
        block_hash.insert(genesis_hash, genesis);
        
        let mut length_hash = HashMap::new();
        length_hash.insert(genesis_hash, 0);
        
        let tip = genesis_hash;
        let longest_length = 0;

        Self {block_hash, length_hash, tip, longest_length}
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {

        // insert block into block_hash
        
        let block_hash = block.hash();
        let block_clone = block.clone();
        self.block_hash.insert(block_hash, block_clone);
        
        // insert length of block into length_hash
        let parent = block.get_parent();
        let parent_length = self.length_hash.get(&parent).unwrap();
        let block_length = parent_length + 1;
        
        self.length_hash.insert(block_hash, block_length);
        
        // update longest_length and tip if necessary
        if block_length > self.longest_length {
            self.longest_length = block_length;
            self.tip = block_hash;
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        // MY CODE
        self.tip
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let tip_hash = self.tip;
        let block = self.block_hash[&tip_hash].clone();
        let mut count = block.get_parent();

        // create vec to store hashes
        let mut output: Vec<H256> = Vec::new();
        
        let zeros: [u8; 32] = [0; 32];
        output.push(tip_hash);

        while count != H256::from(zeros) {
            output.push(count);
            count = self.block_hash[&count].clone().get_parent();
        }
 
        // reverse order of elements in vector
        let length = output.len();
        let half_length = &length / 2;
    
        for i in 0..(half_length) {
            let temp = output[i];
            output[i] = output[&length - 1 - i];
            output[&length - 1 - i] = temp;
        }

        output
        // vec![]
    }

    // get parent_block
    pub fn get_parent_block(&self, parent: H256) -> Block {
        self.block_hash.get(&parent).unwrap().clone()
    }

    // check if block with certain hash present in hashmap
    pub fn is_present(&self, hash: H256) -> bool {

        let output = match self.block_hash.get(&hash) {
            None => false,
            _ => true,
        };

        output
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);

        // MY CODE
        // let block2 = generate_random_block(&block.hash());
        // blockchain.insert(&block2);

        // let block3 = generate_random_block(&block2.hash());
        // blockchain.insert(&block3);

        // let block_side = generate_random_block(&genesis_hash);
        // blockchain.insert(&block_side);

        // assert_eq!(blockchain.tip(), block3.hash());
        assert_eq!(blockchain.tip(), block.hash());

    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
