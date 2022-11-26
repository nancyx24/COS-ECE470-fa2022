use serde::{Serialize, Deserialize};
use ring::digest;
use crate::types::hash::{H256, Hashable};
use super::transaction;
use std::collections::HashMap;
use crate::types::address::Address;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header, // see Header struct below
    data: Content, // see Content struct below
    state: State, // see State struct below
}

// MY CODE
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Header {
    parent: H256, // hash pointer to parent block
    nonce: u32, // random integer used in proof-of-work mining
    difficulty: H256, // threshold in proof-of-work check
    timestamp: u128, // timestamp when block generated
    merkle_root: H256, // Merkle root of data
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Content {
    content_data: Vec<transaction::SignedTransaction>, // actual transactions carried by block
}

// MY CODE
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    // stores <address, (account nonce, account balance)>
    state: HashMap<Address, (u32, u32)>,
}

impl State {
    pub fn new() -> Self {
        let state: HashMap<Address, (u32, u32)> = HashMap::new();
        State{state}
    }

    // insert account into state
    pub fn insert(&mut self, address: Address, nonce: u32, balance: u32) {
        self.state.insert(address, (nonce, balance));
    }

    // check whether state contains an address
    pub fn contains_key(&self, address: Address) -> bool {
        self.state.contains_key(&address)
    }

    // get nonce and balance
    pub fn get(&self, address: Address) -> (u32, u32) {
        self.state.get(&address).unwrap().clone()
    }

    // get content
    pub fn get_state(&self) -> HashMap<Address, (u32, u32)> {
        self.state.clone()
    }
}

// MY CODE
impl Hashable for Header {
    fn hash(&self) -> H256 {
        let serialized = serde_json::to_string(&self).unwrap();
        digest::digest(&digest::SHA256, serialized.as_ref()).into()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        // MY CODE
        self.header.hash()
    }
}

impl Block {
    // return parent
    pub fn get_parent(&self) -> H256 {
        // MY CODE
        self.header.parent.clone()
    }

    // return difficulty
    pub fn get_difficulty(&self) -> H256 {
        // MY CODE
        self.header.difficulty.clone()
    }

    // MY CODE
    // return content
    pub fn get_content(&self) -> Vec<transaction::SignedTransaction> {
        self.data.content_data.clone()
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    // return hashed content as vector of string
    pub fn get_hashed_content(&self) -> Vec<H256> {
        let vector = self.data.content_data.clone();

        let mut signed_tx_vec: Vec<H256> = Vec::new();
        
        for el in vector {
            signed_tx_vec.push(el.hash());
        }

        signed_tx_vec
    }

    pub fn insert_transaction(&mut self, tx: transaction::SignedTransaction) {
        self.data.content_data.push(tx);
    }

    // update the state
    pub fn put_state(&mut self, state: State) {
        self.state = state;
    }
}

// MY CODE

impl Content {
    // return content data
    pub fn get_content_data(&self) -> Vec<transaction::SignedTransaction> {
        self.content_data.clone()
    }
}

// #[cfg(any(test, test_utilities))]

// // MY CODE -- seems like the first function always doesn't work???
// pub fn useless() -> () {
//     println("Useless function")
// }

// MY CODE

pub fn build_content(content_data: Vec<transaction::SignedTransaction>) -> Content {
    Content{ content_data }
}

pub fn build_header(parent: H256, nonce: u32, difficulty: H256, timestamp: u128, merkle_root: H256) -> Header {
    Header{ parent, nonce, difficulty, timestamp, merkle_root }
}

pub fn build_block(header: Header, data: Content, state: State) -> Block {
     Block{ header, data, state }
}

#[cfg(any(test, test_utilities))] // WHAT DOES THIS DO??
pub fn generate_random_block(parent: &H256) -> Block {
    // MY CODE
    let parent = *parent;
    let nonce: u32 = rand::random();
    let zeros: [u8; 32] = [0; 32];
    let difficulty: H256 = [255u8; 32].into();
    let timestamp = 0;
    let content_data: Vec<transaction::SignedTransaction> = Vec::new();
    let data: Content = Content{ content_data };
    let state: State = State::new();

    // merkle root of empty input
    // FOR NOW, BUT NEED TO IMPLEMENT IN MERKLE.RS
    let merkle_root: H256 = H256::from(zeros); 

    let header: Header = Header{ parent, nonce, difficulty, timestamp, merkle_root };
    
    Block{ header, data, state }
}
