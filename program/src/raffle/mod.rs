pub mod feistel;

use feistel::shuffle;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub address: [u8; 20],
    pub start: u64,
    pub end: u64,
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

pub fn draw(num_winners: u64, seed: [u8; 32], entries: &[Entry]) -> Vec<[u8; 20]> {
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
