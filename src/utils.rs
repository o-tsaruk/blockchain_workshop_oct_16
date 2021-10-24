use std::time::{SystemTime, UNIX_EPOCH};
use crate::types::{AccountId, Block, Blockchain, COEFFICIENT_LENGTH, Error, Hash, Target};
use blake2::{Blake2s, Digest};
use ed25519_dalek::{Keypair};
use rand::Rng;

pub fn generate_keypair() -> Keypair {
    Keypair::generate(&mut rand::rngs::OsRng {})
}

pub fn generate_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    let seed: u128 = rng.gen();

    hex::encode(Blake2s::digest(&seed.to_be_bytes()))
}

pub fn generate_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_secs()
}

pub fn hash_to_bits(hash: Hash) -> Hash {
    let exponent;    // size of the hash in bytes
    let coefficient: &str;                      // initial 3 bytes of the hash
    let mut result = String::new();      // 8 digits (4 bytes) long

    let beginning = find_beginning_of_hash(hash.clone());
    let mut new_hash;
    new_hash = hash[beginning..].to_string();

    if new_hash.len() % 2 != 0 {
        new_hash = "0".to_string() + &new_hash;
    }

    let number_of_bytes = new_hash.len()/2;
    exponent = hex::encode(vec![number_of_bytes as u8]);

    coefficient = &new_hash[..COEFFICIENT_LENGTH];  // 3 bytes = 6 digits in hex
    result += exponent.as_str();
    result += coefficient;

    result
}

pub fn find_beginning_of_hash(hash: Hash) -> usize {
    let length_range = hash.len();
    let length: Vec<usize> = (0..length_range).collect();
    let length_iter = length.iter();

    for iter in length_iter {
        if hash.chars().nth(*iter).unwrap() != '0' {
            return *iter
        }
    }

    return 0
}

pub fn check_target(target: Target, hash: Hash) -> bool {
    let result = u64::from_str_radix(&(hash_to_bits(hash.clone())), 16);
    if result.unwrap() < target {
        return true;
    }

    false
}

pub fn mining(block: &mut Block, bc: &Blockchain) -> Result<(), Error> {
    let mut nonce: u128 = 1;
    block.set_nonce(nonce.clone());

    while check_target(bc.current_target.clone(), block.hash.clone().unwrap()) == false {
        nonce += 1;
        block.set_nonce(nonce.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        dbg!(generate_account_id());
    }

    #[test]
    fn test_hash_to_bits() {
        let target = "00000000000000000000000000000000000333a1000000000000000000000000".to_string();
        let result = hash_to_bits(target.clone());
        let target = u64::from_str_radix(&result, 16);
        assert_eq!(result.clone(), "0f0333a1".to_string());
        assert_eq!(target.unwrap(), 251868065)
    }
}
