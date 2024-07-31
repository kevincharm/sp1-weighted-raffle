pub mod feistel;

use feistel::deshuffle;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub address: [u8; 20],
    pub start: u64,
    pub end: u64,
}

fn compute_winner(n: u64, entries: &[Entry], seed: [u8; 32]) -> [u8; 20] {
    let last_entry = entries.last().unwrap();
    let domain = last_entry.end;
    let trunc_seed = u64::from_be_bytes(seed[24..32].try_into().unwrap());
    let winning_index = deshuffle(n, trunc_seed, domain, 4);

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
    assert!(num_winners > 0, "num_winners == 0");
    assert!(
        num_winners <= entries.len() as u64,
        "num_winners > |entries|"
    );

    let mut winners = HashSet::new();
    let mut i = 0u64;
    for _ in 0..num_winners {
        loop {
            let winner = compute_winner(i, entries, seed);
            i += 1;
            if !winners.contains(&winner) {
                winners.insert(winner);
                break;
            }
        }
    }
    winners.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{self, Rng};
    use std::collections::HashSet;

    #[test]
    fn test_draw() {
        let num_entries = 1000;
        let mut entries = vec![];
        for i in 0..num_entries {
            entries.push(Entry {
                address: rand::thread_rng().r#gen(),
                start: i * 10,
                end: i * 10 + 10,
            });
        }
        assert_eq!(
            entries.len(),
            num_entries as usize,
            "rng did not generate {num_entries} distinct entries (rerun the test!)"
        );
        let num_winners = num_entries;
        let seed = rand::thread_rng().r#gen();
        let winners = draw(num_winners, seed, &entries);
        assert_eq!(
            winners.len(),
            num_winners as usize,
            "wrong number of winners drawn"
        );

        // Check winners are distinct
        let mut winners_set = HashSet::new();
        for winner in &winners {
            winners_set.insert(winner);
        }
        assert_eq!(
            winners_set.len(),
            num_winners as usize,
            "winners are not distinct"
        );
    }
}
