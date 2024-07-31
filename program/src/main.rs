#![no_main]
sp1_zkvm::entrypoint!(main);

mod ec;
mod feistel;
use alloy_sol_types::{sol, SolType};
use feistel::shuffle;
use rs_merkle::{Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashSet;

type PublicValuesTuple = sol! {
    // commitment_root, seed, winners_root
    tuple(bytes32,bytes32,bytes32)
};

#[derive(Serialize, Deserialize)]
struct Entry {
    address: [u8; 20],
    start: u64,
    end: u64,
}

#[derive(Serialize, Deserialize)]
struct WeightedRaffleProgramInput {
    seed: [u8; 32],
    entries: Vec<Entry>,
    num_winners: u64,
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

fn compute_winner(n: u64, entries: &[Entry], seed: [u8; 32]) -> [u8; 20] {
    let last_entry = entries.last().unwrap();
    let domain = last_entry.end;
    let trunc_seed = u64::from_be_bytes(seed[24..32].try_into().unwrap());
    let winning_index = shuffle(n, trunc_seed, domain, 4);

    let mut l = 0u64;
    let mut r = entries.len() as u64;
    while l <= r {
        let m = (l + r) / 2;
        let entry = &entries[m as usize];
        if entry.start <= winning_index && winning_index < entry.end {
            return entry.address;
        } else if entry.start > winning_index {
            r = m - 1;
        } else {
            l = m + 1;
        }
    }
    panic!("list exhausted without finding entry");
}

fn draw(num_winners: u64, seed: [u8; 32], entries: &[Entry]) -> Vec<[u8; 20]> {
    assert!(
        num_winners <= entries.len() as u64,
        "num_winners > |entries|"
    );
    let mut winners = HashSet::new();
    let mut i = 0u64;
    for _ in 0..num_winners {
        let mut winner;
        loop {
            winner = compute_winner(i, entries, seed);
            i += 1;
            if !winners.contains(&winner) {
                break;
            }
        }
        winners.insert(winner);
    }
    winners.into_iter().collect()
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
