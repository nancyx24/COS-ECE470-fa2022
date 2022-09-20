use serde::{Serialize,Deserialize};
use ring::signature::{self, Ed25519KeyPair, UnparsedPublicKey, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::Rng;
use super::address::Address;
// crate::types::address::Address;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    // MY CODE

    sender: Address,
    receiver: Address,
    value: i128, // make big for now
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    // MY CODE

    t: Transaction,
    sig: Vec<u8>,
    public_key: Vec<u8>,
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
    let sender_array: [u8; 20] = rand::random();
    let receiver_array: [u8; 20] = rand::random();
    let sender = Address::from(sender_array);
    let receiver = Address::from(receiver_array);
    let value = rand::random();

    Transaction{sender: sender, receiver: receiver, value: value}
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
