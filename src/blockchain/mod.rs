use crate::types::hash::H256;
use crate::types::hash::Hashable;
use std::collections::HashMap;
use super::types::block::{self, Block, Header, Content, State};
use super::types::transaction::{self, Transaction, SignedTransaction};
use crate::types::address::Address;
use ring::signature::KeyPair;
use ring::signature::Ed25519KeyPair;

pub struct Blockchain {
    block_hash: HashMap<H256, Block>, // key = hash, value = block
    length_hash: HashMap<H256, u128>, // key = hash, value = length of block
    tip: H256, // last block's hash in longest chain
    longest_length: u128, // length of longest chain
    blockchain_state: HashMap<H256, State>, // key = hash of block, value = state
}

// structure to store received valid transactions not included blockchain yet
pub struct Mempool {
    mem_pool: HashMap<H256, SignedTransaction>,
}

impl Mempool {
    // constructor
    pub fn new() -> Self {
        Self {mem_pool: HashMap::new()}
    }

    // checks if present
    pub fn is_present(&self, hash: H256) -> bool {

        let output = match self.mem_pool.get(&hash) {
            None => false,
            _ => true,
        };

        output
    }

    // get transaction
    pub fn get_transaction(&self, hash: H256) -> SignedTransaction {
        self.mem_pool.get(&hash).unwrap().clone()
    }

    // get mem_pool
    pub fn get_mempool(&self) -> HashMap<H256, SignedTransaction> {
        self.mem_pool.clone()
    }

    // insert transaction into hashmap
    pub fn insert(&mut self, hash: H256, trans: &SignedTransaction) {
        let transaction_clone = trans.clone();
        self.mem_pool.insert(hash, transaction_clone); // ASK ABOUT THIS

        // let block_hash = block.hash();
        // let block_clone = block.clone();
        // self.block_hash.insert(block_hash, block_clone);
    }

    // remove transaction from hashmap
    pub fn remove(&mut self, hash: H256) {
        self.mem_pool.remove(&hash);
    }
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new(key_pair: Ed25519KeyPair) -> Self {
        // MY CODE

        // create genesis block
        let zeros: [u8; 32] = [0; 32];
        let parent: H256 = H256::from(zeros); // total 64
        let difficulty = hex_literal::hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        // hex_literal::hex!("00000effffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into(); // five 0s and all fs
        // H256::from([1u8; 32])
        let timestamp = 0;

        // generate random key_pair and transaction -- CHECK THIS
        let receiver = Address::from_public_key_bytes(key_pair.public_key().as_ref());
        let value = 1000000000; // I chose this
        let nonce = 1000000; // I chose this

        let public_key = key_pair.public_key().as_ref().to_vec();
        let transaction = Transaction::new(receiver, value, nonce);
        let sig = transaction::sign(&transaction, &key_pair).as_ref().to_vec();
        let signed_transaction = SignedTransaction::new(transaction, sig, public_key);

        // make state
        let mut state = State::new();
        state.insert(receiver, nonce, value);

        // make content
        let mut content_data: Vec<SignedTransaction> = Vec::new();
        content_data.push(signed_transaction);
        let data: Content = block::build_content(content_data);

        // merkle root of empty input
        // FOR NOW, BUT NEED TO IMPLEMENT IN MERKLE.RS
        let merkle_root: H256 = H256::from(zeros); 
        
        let header: Header = block::build_header(parent, nonce, difficulty, timestamp, merkle_root);
        let genesis: Block = block::build_block(header, data, state.clone());
        let genesis_hash = genesis.hash();

        // make blockchain state
        let mut blockchain_state: HashMap<H256, State> = HashMap::new();
        blockchain_state.insert(genesis_hash.clone(), state.clone());

        // create contents for Blockchain
        let mut block_hash = HashMap::new();
        block_hash.insert(genesis_hash, genesis);
        
        let mut length_hash = HashMap::new();
        length_hash.insert(genesis_hash, 0);
        
        let tip = genesis_hash;
        let longest_length = 0;

        Self {block_hash, length_hash, tip, longest_length, blockchain_state}
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
        // key_pair for 6000 port (doesn't matter)
        let key_pair = Ed25519KeyPair::from_seed_unchecked(&[0; 32]).unwrap();
        let mut blockchain = Blockchain::new(key_pair);
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
