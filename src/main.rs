use blake2::{Digest};
use blockchain_workshop::traits::Hashable;
use blockchain_workshop::types::{COEFFICIENT_LENGTH, EXPECTED_TIME, MAX_COMPACT_FORM, Transaction, TransactionData};
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};
use blockchain_workshop::utils::{generate_account_id, generate_timestamp, hash_to_bits};
use std::time::{SystemTime, Duration, UNIX_EPOCH};

fn main() {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch.as_secs());
}
