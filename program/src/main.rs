#[cfg(not(test))]
sp1_zkvm::entrypoint!(main);

mod raffle;
use alloy_sol_types::{sol, SolType};
use raffle::{draw, Entry};
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

type PublicValuesTuple = sol! {
    // commitment_root, seed, winners_root
    tuple(bytes32,bytes32,bytes32)
};

#[derive(Serialize, Deserialize)]
struct WeightedRaffleProgramInput {
    seed: [u8; 32],
    entries: Vec<Entry>,
    num_winners: u64,
}

#[derive(Clone)]
pub struct Keccak256Algorithm;

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

    // Compute Merkle root of original commitment
    // Leaves in the commitment tree are the hashes of the entries i.e. H(address || start || end)
    let commit_leaves: Vec<[u8; 32]> = input
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
    let commit_tree = MerkleTree::<Keccak256Algorithm>::from_leaves(&commit_leaves);
    let commit_root = commit_tree.root().ok_or("failed to compute root").unwrap();

    // Draw winners & commit winners' Merkle root
    // Leaves in the winners' Merkle root are the hashes of the winners i.e. H(address)
    let winners = draw(input.num_winners, input.seed, &input.entries);
    let winners_leaves: Vec<[u8; 32]> = winners
        .into_iter()
        .map(|address| {
            let mut hasher = Keccak256::new();
            hasher.update(address);
            hasher.finalize().into()
        })
        .collect();
    let winners_tree = MerkleTree::<Keccak256Algorithm>::from_leaves(&winners_leaves);
    let winners_root = winners_tree.root().ok_or("failed to compute root").unwrap();

    println!("cycle-tracker-end: main");

    // Encode the public values of the program.
    let bytes = PublicValuesTuple::abi_encode(&(commit_root, input.seed, winners_root));
    // Commit to the public values of the program.
    sp1_zkvm::io::commit_slice(&bytes);
}
