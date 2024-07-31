use sha3::{Digest, Keccak256};

// Babylonian sqrt
fn sqrt(s: u64) -> u64 {
    println!("cycle-tracker-start: sqrt");
    let mut z = 0;
    if s > 3 {
        z = s;
        let mut x = s / 2 + 1;
        while x < z {
            z = x;
            x = (s / x + x) / 2;
        }
    } else if s != 0 {
        z = 1;
    }
    println!("cycle-tracker-end: sqrt");
    z
}

// Take next perfect square unless n is already one
fn next_perfect_square(n: u64) -> u64 {
    println!("cycle-tracker-start: next-perfect-square");
    let sqrt_n = sqrt(n);
    if sqrt_n.pow(2) == n {
        return n;
    }
    let out = (sqrt_n + 1).pow(2);
    println!("cycle-tracker-end: next-perfect-square");
    out
}

fn f(x: u64, i: u64, seed: u64, modulus: u64) -> u64 {
    println!("cycle-tracker-start: round-func");
    let mut keccak = Keccak256::new();
    keccak.update(x.to_le_bytes());
    keccak.update(i.to_le_bytes());
    keccak.update(seed.to_le_bytes());
    keccak.update(modulus.to_le_bytes());
    let output: [u8; 32] = keccak.finalize().into(); // 64b
    let trunc_output = output[24..32].try_into().unwrap();
    let out = u64::from_be_bytes(trunc_output);
    println!("cycle-tracker-end: round-func");
    out
}

#[allow(dead_code)]
pub fn shuffle(_x: u64, seed: u64, domain: u64, rounds: u64) -> u64 {
    assert!(domain != 0, "modulus must be > 0");
    assert!(_x < domain, "x too large");
    assert!((rounds & 1) == 0, "rounds must be even");

    println!("cycle-tracker-start: shuffle");
    let mut x = _x;
    let h = sqrt(next_perfect_square(domain));
    loop {
        let mut l = x % h;
        let mut r = x / h;
        for i in 0..rounds {
            let next_r = (l + f(r, i, seed, domain)) % h;
            l = r;
            r = next_r;
        }
        x = h * r + l;
        if x < domain {
            break;
        }
    }
    println!("cycle-tracker-end: shuffle");
    x
}

#[allow(dead_code)]
pub fn deshuffle(_x_prime: u64, seed: u64, domain: u64, rounds: u64) -> u64 {
    assert!(domain != 0, "modulus must be > 0");
    assert!(_x_prime < domain, "x too large");
    assert!((rounds & 1) == 0, "rounds must be even");

    println!("cycle-tracker-start: deshuffle");
    let mut x_prime = _x_prime;
    let h = sqrt(next_perfect_square(domain));
    loop {
        let mut l = x_prime % h;
        let mut r = x_prime / h;
        for i in 0..rounds {
            let next_l = (r + h - (f(l, rounds - i - 1, seed, domain) % h)) % h;
            r = l;
            l = next_l;
        }
        x_prime = h * r + l;
        if x_prime < domain {
            break;
        }
    }
    println!("cycle-tracker-end: deshuffle");
    x_prime
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::collections::HashSet;

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(0), 0);
        assert_eq!(sqrt(1), 1);
        assert_eq!(sqrt(2), 1);
        assert_eq!(sqrt(3), 1);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(5), 2);
        assert_eq!(sqrt(6), 2);
        assert_eq!(sqrt(7), 2);
        assert_eq!(sqrt(8), 2);
        assert_eq!(sqrt(9), 3);
    }

    #[test]
    fn test_next_perfect_square() {
        assert_eq!(next_perfect_square(0), 0);
        assert_eq!(next_perfect_square(1), 1);
        assert_eq!(next_perfect_square(2), 4);
        assert_eq!(next_perfect_square(3), 4);
        assert_eq!(next_perfect_square(4), 4);
        assert_eq!(next_perfect_square(5), 9);
        assert_eq!(next_perfect_square(6), 9);
        assert_eq!(next_perfect_square(7), 9);
        assert_eq!(next_perfect_square(8), 9);
        assert_eq!(next_perfect_square(9), 9);
    }

    #[test]
    fn test_invertibility() {
        let seed = rand::thread_rng().gen();
        let domain = 1000;
        let rounds = 4;
        for x in 0..domain {
            let x_prime = shuffle(x, seed, domain, rounds);
            let x_prime_prime = deshuffle(x_prime, seed, domain, rounds);
            assert_eq!(x, x_prime_prime);
        }
    }

    #[test]
    fn test_bijectivity() {
        let mut set = HashSet::<u64>::new();
        let seed = rand::thread_rng().gen();
        let domain = 1000;
        let rounds = 4;
        for x in 0..domain {
            let x_prime = shuffle(x, seed, domain, rounds);
            set.insert(x_prime);
        }
        assert_eq!(set.len(), domain as usize, "not bijective");
    }
}
