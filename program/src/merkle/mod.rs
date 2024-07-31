use crate::raffle::{draw, Entry};
use rs_merkle::{Hasher, MerkleTree};
use sha3::{Digest, Keccak256};

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

pub fn get_merkle_root(leaves: Vec<[u8; 32]>) -> [u8; 32] {
    let winners_tree = MerkleTree::<Keccak256Algorithm>::from_leaves(&leaves);
    winners_tree.root().ok_or("failed to compute root").unwrap()
}

// Compute Merkle root of original commitment
// Leaves in the commitment tree are the hashes of the entries i.e. H(address || start || end)
pub fn get_commitment_root(entries: &[Entry]) -> [u8; 32] {
    assert!(entries.len() >= 2, "<2 entries");

    let commit_leaves: Vec<[u8; 32]> =
        entries
            .iter()
            .fold(vec![], |mut acc: Vec<[u8; 32]>, entry| {
                // Invariant: first entry must start at 0
                if acc.is_empty() {
                    assert!(entry.start == 0, "first entry must start at 0");
                }
                // Invariant: weight must be positive
                assert!(entry.start < entry.end, "invalid entry");

                // Invariant: entries must be adjacent segments
                if !acc.is_empty() {
                    let last_entry = &entries[acc.len() - 1];
                    assert!(last_entry.end == entry.start, "non-adjacent entries");
                }

                // Hash leaf = H(address || start || end)
                let mut hasher = Keccak256::new();
                hasher.update(entry.address);
                hasher.update(entry.start.to_be_bytes());
                hasher.update(entry.end.to_be_bytes());
                let leaf: [u8; 32] = hasher.finalize().into();
                acc.push(leaf);
                acc
            });
    get_merkle_root(commit_leaves)
}

// Draw winners & commit winners' Merkle root
// Leaves in the winners' Merkle root are the hashes of the winners i.e. H(address)
pub fn get_winners_root(num_winners: u64, seed: [u8; 32], entries: &[Entry]) -> [u8; 32] {
    let winners = draw(num_winners, seed, entries);
    let winners_leaves: Vec<[u8; 32]> = winners
        .into_iter()
        .map(|address| {
            let mut hasher = Keccak256::new();
            hasher.update(address);
            hasher.finalize().into()
        })
        .collect();
    get_merkle_root(winners_leaves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "<2 entries")]
    fn test_get_commitment_root_asserts_min_entries() {
        let entries_0 = vec![];
        get_commitment_root(&entries_0);

        let entries_1 = vec![Entry {
            address: [1; 20],
            start: 0,
            end: 10,
        }];
        get_commitment_root(&entries_1);
    }

    #[test]
    #[should_panic(expected = "first entry must start at 0")]
    fn test_get_commitment_root_asserts_first_entry_valid() {
        let entries = vec![
            Entry {
                address: [1; 20],
                start: 1, // <-- invalid (must start at 0)
                end: 10,
            },
            Entry {
                address: [2; 20],
                start: 10,
                end: 20,
            },
        ];
        get_commitment_root(&entries);
    }

    #[test]
    #[should_panic(expected = "invalid entry")]
    fn test_get_commitment_root_asserts_weight_nonzero() {
        let entries = vec![
            Entry {
                address: [1; 20],
                start: 0,
                end: 0, // <-- weight = 0 - 0 = 0
            },
            Entry {
                address: [2; 20],
                start: 0,
                end: 10,
            },
        ];
        get_commitment_root(&entries);
    }

    #[test]
    #[should_panic(expected = "invalid entry")]
    fn test_get_commitment_root_asserts_weight_positive() {
        let entries = vec![
            Entry {
                address: [1; 20],
                start: 0,
                end: 10,
            },
            Entry {
                address: [2; 20],
                start: 10,
                end: 9, // <-- weight = 9 - 10 = -1
            },
        ];
        get_commitment_root(&entries);
    }

    #[test]
    #[should_panic(expected = "non-adjacent entries")]
    fn test_get_commitment_root_asserts_adjacentness() {
        let entries = vec![
            Entry {
                address: [1; 20],
                start: 0,
                end: 11, // <-- non-adjacent (ends after entry@1)
            },
            Entry {
                address: [2; 20],
                start: 10, // <-- non-adjacent (starts before entry@0)
                end: 20,
            },
        ];
        get_commitment_root(&entries);
    }
}
