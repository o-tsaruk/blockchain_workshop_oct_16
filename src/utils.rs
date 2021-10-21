use crate::types::{AccountId, Balance, Block, Blockchain, Error, Transaction, TransactionData};
use blake2::{Blake2s, Digest};
use ed25519_dalek::{Keypair, Signer};
use rand::Rng;
use crate::traits::Hashable;

pub fn generate_keypair() -> Keypair {
    Keypair::generate(&mut rand::rngs::OsRng {})
}

pub fn generate_account_id() -> AccountId {
    let mut rng = rand::thread_rng();
    let seed: u128 = rng.gen();

    hex::encode(Blake2s::digest(&seed.to_be_bytes()))
}

// pub fn append_block(bc: &mut Blockchain, nonce: u128) -> Block {
//     let mut block = Block::new(bc.get_last_block_hash());
//     let tx_create_account =
//         Transaction::new(TransactionData::CreateAccount(generate_account_id()), None);
//     block.set_nonce(nonce);
//     block.add_transaction(tx_create_account);
//     let block_clone = block.clone();
//
//     assert!(bc.append_block(block).is_ok());
//
//     block_clone
// }
pub fn create_block(bc: &mut Blockchain, nonce: u128, user1_id: AccountId) -> Block {
    let mut block = Block::new(bc.get_last_block_hash());

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    //let user1_id = generate_account_id();
    let mut tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some(user1_id.clone()));

    tx_create_account_user1.signature =
        Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());

    block.set_nonce(nonce);
    block.add_transaction(tx_create_account_user1.clone());

    block.clone()
}

pub fn create_block_and_tx(bc: &mut Blockchain, nonce: u128, mint_amount: Vec<Balance>, tx_amount: Balance,
    user1_id: AccountId, user2_id: AccountId) -> Block {

    let mut block = Block::new(bc.get_last_block_hash());
    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;

    let mut tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some(user1_id.clone()));

    let tx_mint_init_supply_user1:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: user1_id.clone(),
            amount: mint_amount[0],
        },
        None,
    );

    tx_create_account_user1.signature =
        Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());

    let user2_keypair = generate_keypair();
    let user2_pk = user2_keypair.public;

    let mut tx_create_account_user2 =
        Transaction::new(TransactionData::CreateAccount(user2_id.clone(), user2_pk),
                         Some(user2_id.clone()));

    let tx_mint_init_supply_user2:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: user2_id.clone(),
            amount: mint_amount[1],
        },
        None,
    );

    tx_create_account_user2.signature =
        Some(user2_keypair.sign(tx_create_account_user2.hash().as_bytes()).to_bytes());

    let mut tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: user2_id.clone(),
            amount: tx_amount,
        },
        Some(user1_id.clone()),
    );

    tx_transfer1.signature =
        Some(user1_keypair.sign(tx_transfer1.hash().as_bytes()).to_bytes());

    block.set_nonce(nonce);
    block.add_transaction(tx_create_account_user1.clone());
    block.add_transaction(tx_mint_init_supply_user1.clone());
    block.add_transaction(tx_create_account_user2.clone());
    block.add_transaction(tx_mint_init_supply_user2.clone());
    block.add_transaction(tx_transfer1.clone());

    block.clone()
}

pub fn append_block_with_tx(
    bc: &mut Blockchain,
    nonce: u128,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let mut block = Block::new(bc.get_last_block_hash());
    block.set_nonce(nonce);

    for tx in transactions {
        block.add_transaction(tx);
    }

    bc.append_block(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        dbg!(generate_account_id());
    }
}
