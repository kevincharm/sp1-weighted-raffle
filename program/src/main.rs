#![no_main]
sp1_zkvm::entrypoint!(main);

mod ec;
mod feistel;
use alloy_sol_types::{sol, SolType};
use feistel::shuffle;
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

type PublicValuesTuple = sol! {
    tuple(bytes32,)
};

#[derive(Serialize, Deserialize)]
struct Entry {
    address: [u8; 20],
    start: u64,
    end: u64,
}

#[derive(Serialize, Deserialize)]
struct WeightedRaffleProgramInput {
    entries: Vec<Entry>,
}

#[derive(Clone)]
struct Keccak256Algorithm(Keccak256);

impl Hasher for Keccak256Algorithm {
    type Hash = [u8; 32];
    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

pub fn main() {
    let input = sp1_zkvm::io::read::<WeightedRaffleProgramInput>();

    println!("cycle-tracker-start: main");
    let leaves: Vec<[u8; 32]> = input
        .entries
        .iter()
        .map(|entry| {
            let mut hasher = Keccak256::new();
            hasher.update(entry.address);
            hasher.update(entry.start.to_be_bytes());
            hasher.update(entry.end.to_be_bytes());
            hasher.finalize().into()
        })
        .collect();
    let tree = MerkleTree::<Keccak256Algorithm>::from_leaves(&leaves);
    let root = tree.root().ok_or("failed to get root").unwrap();
    println!("cycle-tracker-end: main");

    // Encode the public values of the program.
    let bytes = PublicValuesTuple::abi_encode(&(root,));
    // Commit to the public values of the program.
    sp1_zkvm::io::commit_slice(&bytes);
}
