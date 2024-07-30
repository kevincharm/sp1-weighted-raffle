use tiny_keccak::{Hasher, Keccak};

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
    let mut keccak = Keccak::v256();
    keccak.update(&x.to_le_bytes());
    keccak.update(&i.to_le_bytes());
    keccak.update(&seed.to_le_bytes());
    keccak.update(&modulus.to_le_bytes());
    let mut output = [0u8; 32]; // 64b
    keccak.finalize(&mut output);
    let trunc_output = output[24..32].try_into().unwrap();
    let out = u64::from_be_bytes(trunc_output);
    println!("cycle-tracker-end: round-func");
    out
}

pub fn shuffle(_x: u64, seed: u64, domain: u64, rounds: u64) -> u64 {
    println!("cycle-tracker-start: shuffle");
    let mut x = _x;
    let h = sqrt(next_perfect_square(domain));
    loop {
        let mut l = x % h;
        let mut r = x / h;
        for i in 0..rounds {
            let hash = f(r, i, seed, domain);
            let next_r = (l + hash) % h;
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
