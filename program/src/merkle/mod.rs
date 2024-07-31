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
    let winners_tree: MerkleTree<Keccak256Algorithm> =
        MerkleTree::<Keccak256Algorithm>::from_leaves(&leaves);
    winners_tree.root().ok_or("failed to compute root").unwrap()
}

fn is_ordered(a: &[u8; 20], b: &[u8; 20]) -> bool {
    for i in 0..20 {
        match (a[i], b[i]) {
            (left, right) if left < right => return true,
            (left, right) if left > right => return false,
            _ => continue,
        }
    }
    false
}

// Compute Merkle root of original commitment
// Leaves in the commitment tree are the hashes of the entries i.e. H(address || start || end)
pub fn get_commitment_root(entries: &[Entry]) -> [u8; 32] {
    assert!(entries.len() >= 2, "<2 entries");

    let commit_leaves = entries
        .iter()
        .fold(vec![] as Vec<[u8; 32]>, |mut acc, entry| {
            // Invariant: first entry must start at 0
            if acc.is_empty() {
                assert!(entry.start == 0, "first entry must start at 0");
            }
            // Invariant: weight must be positive
            assert!(entry.start < entry.end, "invalid entry");

            if !acc.is_empty() {
                let last_entry = &entries[acc.len() - 1];
                // Invariant: entries must be adjacent segments
                assert!(last_entry.end == entry.start, "non-adjacent entries");
                // Invariant: addresses are identities and must be distinct
                assert!(
                    is_ordered(&last_entry.address, &entry.address),
                    "entries must be ordered (asc) by addresses"
                );
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

    #[test]
    #[should_panic(expected = "entries must be ordered (asc) by addresses")]
    fn test_get_commitment_root_asserts_no_duplicates() {
        let entries = vec![
            Entry {
                address: [0x11; 20],
                start: 0,
                end: 10,
            },
            Entry {
                address: [0x11; 20], // <-- duplicate
                start: 10,
                end: 20,
            },
        ];
        get_commitment_root(&entries);
    }

    #[test]
    #[should_panic(expected = "entries must be ordered (asc) by addresses")]
    fn test_get_commitment_root_asserts_ordering() {
        let entries = vec![
            Entry {
                address: [0x22; 20],
                start: 0,
                end: 10,
            },
            Entry {
                address: [0x11; 20], // <-- ordered descendingly
                start: 10,
                end: 20,
            },
        ];
        get_commitment_root(&entries);
    }

    #[test]
    fn test_merkle_odd() {
        let leaves = vec![[0x11u8; 20], [0x22; 20], [0x33; 20]]
            .into_iter()
            .map(|address| {
                let mut hasher = Keccak256::new();
                hasher.update(address);
                hasher.finalize().into()
            })
            .collect();
        println!("leaves = {:02x?}", leaves);
        println!("root = {:02x?}", get_merkle_root(leaves));
    }
}
