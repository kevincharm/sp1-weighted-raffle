use std::path::PathBuf;

use alloy_sol_types::{sol, SolType};
use clap::Parser;
use hex;
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
///
/// This file is generated by running `cargo prove build` inside the `program` directory.
pub const ELF: &[u8] = include_bytes!("../../../program/elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the prove command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ProveArgs {
    #[clap(long, default_value = "1")]
    x: u64,
    #[clap(long, default_value = "10")]
    domain: u64,
    #[clap(long, default_value = "16045690984833335023")]
    seed: u64,
    #[clap(long, default_value = "4")]
    rounds: u64,

    #[clap(long, default_value = "false")]
    evm: bool,
}

/// The public values encoded as a tuple that can be easily deserialized inside Solidity.
type PublicValuesTuple = sol! {
    tuple(uint64,)
};

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = ProveArgs::parse();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ELF);

    println!("x: {}", args.x);
    println!("domain: {}", args.domain);
    println!("seed: {}", args.seed);
    println!("rounds: {}", args.rounds);
    // Setup the inputs.

    for i in 0u64..10 {
        let mut stdin = SP1Stdin::new();
        // stdin.write(&args.x);
        stdin.write(&i);
        stdin.write(&args.domain);
        stdin.write(&args.seed);
        stdin.write(&args.rounds);

        if args.evm {
            // Generate the proof.
            let proof = client
                .prove(&pk, stdin)
                .plonk()
                .run()
                .expect("failed to generate proof");
            create_plonk_fixture(&proof, &vk);
        } else {
            // Generate the proof.
            // let proof = client
            //     .prove(&pk, stdin)
            //     .run()
            //     .expect("failed to generate proof");
            // let proof = proof.public_values;
            let (public_values, _) = client.execute(ELF, stdin).run().unwrap();
            let (x_prime,) =
                PublicValuesTuple::abi_decode(public_values.as_slice(), false).unwrap();
            println!("Successfully generated proof!");
            println!("x = {}, x_prime = {}\n", i, x_prime);

            // Verify the proof.
            // client.verify(&proof, &vk).expect("failed to verify proof");
        }
    }
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1ProofFixture {
    x_prime: u64,
    vkey: String,
    public_values: String,
    proof: String,
}

/// Create a fixture for the given proof.
fn create_plonk_fixture(proof: &SP1ProofWithPublicValues, vk: &SP1VerifyingKey) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let (x_prime,) = PublicValuesTuple::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-ot-end.
    let fixture = SP1ProofFixture {
        x_prime,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values whicha are publically commited to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join("fixture.json"),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
