#![cfg_attr(not(test), no_main)]
#[cfg(not(test))]
sp1_zkvm::entrypoint!(main);

mod merkle;
mod raffle;
use alloy_sol_types::{sol, SolType};
use merkle::{get_commitment_root, get_winners_root};
use raffle::Entry;
use serde::{Deserialize, Serialize};

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

pub fn main() {
    let input = sp1_zkvm::io::read::<WeightedRaffleProgramInput>();

    println!("cycle-tracker-start: main");

    let commit_root = get_commitment_root(&input.entries);
    let winners_root = get_winners_root(input.num_winners, input.seed, &input.entries);

    println!("cycle-tracker-end: main");

    // Encode the public values of the program.
    let bytes = PublicValuesTuple::abi_encode(&(commit_root, input.seed, winners_root));
    // Commit to the public values of the program.
    sp1_zkvm::io::commit_slice(&bytes);
}
