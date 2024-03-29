use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, UnparsedPublicKey, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::Rng;
use ring::digest;
use std::convert::TryInto;
use super::address::Address;
use crate::types::hash::H256;
use crate::types::hash::Hashable;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    // MY CODE

    // account-based model transaction
    receiver: Address,
    value: u32, // make big for now
    nonce: u32, 
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    // MY CODE

    t: Transaction,
    sig: Vec<u8>,
    public_key: Vec<u8>,
}

impl Transaction {
    // new function
    pub fn new(receiver: Address, value: u32, nonce: u32)-> Transaction {
        Transaction {receiver, value, nonce}
    }

    pub fn get_value(&self) -> u32 {
        self.value.clone()
    }

    pub fn get_nonce(&self) -> u32 {
        self.nonce.clone()
    }

    pub fn get_receiver(&self) -> Address {
        self.receiver.clone()
    }
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        // MY CODE
        // Miraculously compiled?

        // serialize transaction as slice of bytes
        let serialized = serde_json::to_string(&self).unwrap(); // &[u8]
        digest::digest(&digest::SHA256, serialized.as_ref()).into()

        // let serialized = serde_json::to_string(&self.t).unwrap().as_ref();
        // let hash: [u8] = digest::digest(&digest::SHA256, serialized).as_ref();
        // let hash_array = hash.try_into().unwrap();
        // H256::from(hash_array)
    }
}

impl SignedTransaction {
    // new function
    pub fn new(t: Transaction, sig: Vec<u8>, public_key: Vec<u8>) -> SignedTransaction {
        SignedTransaction {t, sig, public_key}
    }

    // get transaction
    pub fn get_t(&self) -> Transaction {
        self.t.clone()
    }
    // get public key
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }
    // get signature
    pub fn get_sig(&self) -> Vec<u8> {
        self.sig.clone()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // MY CODE

    // serialize transaction as slice of bytes
    let serialized = serde_json::to_string(&t).unwrap();

    // sign with key
    key.sign(serialized.as_bytes())
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    // MY CODE

    // makes unparsed public key object
    let peer_public_key = UnparsedPublicKey::new(&signature::ED25519, public_key);
    // serializes transaction
    let serialized = serde_json::to_string(&t).unwrap();
    // verifies
    peer_public_key.verify(serialized.as_ref(), signature.as_ref()).is_ok()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    // MY CODE

    // makes random sender, receiver, value
    let receiver_array: [u8; 20] = rand::random();
    let receiver = Address::from(receiver_array);
    let value = rand::random();
    let nonce = rand::random();

    Transaction{receiver: receiver, value: value, nonce: nonce}
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
