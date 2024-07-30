#![no_main]
sp1_zkvm::entrypoint!(main);

mod feistel;
use feistel::shuffle;

use alloy_sol_types::{sol, SolType};

type PublicValuesTuple = sol! {
    tuple(uint64,)
};

pub fn main() {
    let x = sp1_zkvm::io::read::<u64>();
    let domain = sp1_zkvm::io::read::<u64>();
    let seed = sp1_zkvm::io::read::<u64>();
    let rounds = sp1_zkvm::io::read::<u64>();

    println!("cycle-tracker-start: main");
    let x_prime = shuffle(x, seed, domain, rounds);
    println!("cycle-tracker-end: main");

    // Encode the public values of the program.
    let bytes = PublicValuesTuple::abi_encode(&(x_prime,));
    // Commit to the public values of the program.
    sp1_zkvm::io::commit_slice(&bytes);
}
