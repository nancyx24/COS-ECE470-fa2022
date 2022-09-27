use super::hash::{Hashable, H256};
use ring::digest;
use std::convert::TryInto;
use std::iter::Iterator;

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    tree: Vec<[u8; 32]>, // vector storing data
    tree_size: Vec<usize>, // vector storing size of each 'row' of tree
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        // define variables for merkle tree struct
        let mut tree: Vec<[u8; 32]> = Vec::new();
        let mut tree_size: Vec<usize> = Vec::new();
        let mut length: u128 = data.len().try_into().unwrap();

        // push data to tree after hashing it first
        for n in 0..length {
            let hash = data[n as usize].hash();
            tree.push(hash.as_ref().try_into().unwrap());
        }

        // counter for calculating hashes
        let mut start_index = 0;

        while length > 1 {
            // duplicate last element if row odd
            // adds length of current row to tree_length
            let mut row_length = 0;

            if length % 2 == 1 {
                tree.push(tree[tree.len() - 1]);
                row_length = length + 1;
            }
            else {
                row_length = length;
            }

            // adds row_length into vector of tree sizes
            tree_size.push(row_length.try_into().unwrap());

            // push data to tree
            for m in (start_index..start_index + row_length).step_by(2) {
                // hash in sections
                let mut ctx = digest::Context::new(&digest::SHA256);
                ctx.update(&tree[m as usize]);
                ctx.update(&tree[(m + 1) as usize]);
                let hash = ctx.finish();

                // push hash into tree
                tree.push(hash.as_ref()[..].try_into().unwrap())
            }

            // goes to next iteration
            start_index = start_index + row_length;
            length = row_length / 2;
        }

        MerkleTree {tree: tree, tree_size: tree_size}
    }

    pub fn root(&self) -> H256 {
        // last element from tree is root
        let length = self.tree.len();
        let root = self.tree[length - 1];
        H256::from(root)
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        // vec of proof
        let mut proof: Vec<H256> = Vec::new();

        // counters
        let mut current_index: usize = 0; // current index of loop
        let mut current_row: usize = 0; // current row of loop
        let mut new_index: usize = index; // index on current row

        while current_index < self.tree.len() - 1 {
            if new_index % 2 == 0 {
                // if even, then sibling is entry on right
                proof.push(H256::from(self.tree[(current_index + index + 1) as usize]));
            }
            else {
                // if odd, then sibling is entry on left
                proof.push(H256::from(self.tree[(current_index + index - 1) as usize]));
            }

            // updates counters
            current_index = current_index + self.tree_size[current_row];
            new_index = index / 2; // integer division
            current_row = current_row + 1;
        }

        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    if index >= leaf_size {
        panic!("Input index out of range!");
    }

    let mut new_index = index; // counter
    let mut hash = *datum; // hash data

    for n in 0..proof.len() {
        let mut ctx = digest::Context::new(&digest::SHA256);

        // if even, left concatenate
        if new_index % 2 == 0 {
            ctx.update(&hash.as_ref());
            ctx.update(&proof[n].as_ref());
        }
        else { // if odd, right concatenate
            ctx.update(&proof[n].as_ref());
            ctx.update(&hash.as_ref());
        }

        let completed = ctx.finish();

        hash = H256::from(completed);

        new_index = new_index / 2; // integer division
    }

    *root == hash
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
                
                // if have above repeated 4 times, get hex in merkle_root
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // (hex!("88d7bd4a65271dd8c891a067cff37d67904e11f0844323960915463384a8d504")).into()

        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![
                    hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into(),
                    // hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920").into(),
                    // hex!("a4efcefb722cea4fc3a4b863e1d7ad7025e726d79ff1c74b5f745ad1dca874ca").into(),
                   ]
        );

        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
