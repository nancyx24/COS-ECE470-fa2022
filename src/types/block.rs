use serde::{Serialize, Deserialize};
use ring::digest;
use crate::types::hash::{H256, Hashable};
use super::transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    header: Header, // see Header struct below
    data: Content, // see Content struct below
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
        self.header.parent
    }

    // return difficulty
    pub fn get_difficulty(&self) -> H256 {
        // MY CODE
        self.header.difficulty
    }
}

#[cfg(any(test, test_utilities))]
// MY CODE
pub fn build_header(parent: H256, nonce: u32, difficulty: H256, timestamp: u128, merkle_root: H256) -> Header {
    Header{ parent, nonce, difficulty, timestamp, merkle_root }
}

pub fn build_content(content_data: Vec<transaction::SignedTransaction>) -> Content {
    Content{ content_data }
}

pub fn build_block(header: Header, data: Content) -> Block {
    Block{ header, data }
}

pub fn generate_random_block(parent: &H256) -> Block {
    // MY CODE
    let parent = *parent;
    let nonce: u32 = rand::random();
    let zeros: [u8; 32] = [0; 32];
    let difficulty: H256 = H256::from(zeros);
    let timestamp = 0;
    let content_data: Vec<transaction::SignedTransaction> = Vec::new();
    let data: Content = Content{ content_data };

    // merkle root of empty input
    // FOR NOW, BUT NEED TO IMPLEMENT IN MERKLE.RS
    let merkle_root: H256 = H256::from(zeros); 

    let header: Header = Header{ parent, nonce, difficulty, timestamp, merkle_root };
    
    Block{ header, data }
}
