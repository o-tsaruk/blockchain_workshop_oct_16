use ed25519_dalek::Signer;
use blockchain_workshop::traits::{Hashable, WorldState};
use blockchain_workshop::types::{Block, Blockchain, Transaction, TransactionData};
use blockchain_workshop::utils::{generate_account_id, generate_keypair, mining};
mod common;
use common::{append_block_with_tx, create_block, create_block_and_tx};

#[test]
fn test_create_blockchain() {
    let bc = &mut Blockchain::new();

    let block1 = create_block_and_tx(bc, vec![10,0], 5,
                                     "bob".to_string(), "alice".to_string());

    assert!(bc.append_block(block1.clone()).is_ok());

    let block2 = create_block(bc,generate_account_id());
    assert!(bc.append_block(block2.clone()).is_ok());

    let block3 = create_block(bc,generate_account_id());
    assert!(bc.append_block(block3.clone()).is_ok());

    assert_eq!(bc.get_last_block_hash(), block3.hash.clone());
    assert!(bc.validate().is_ok());
}

#[test]
fn test_state_rollback_works() {
    let mut bc = Blockchain::new();

    // accounts data
    let satoshi_keypair = generate_keypair();
    let satoshi_id = "satoshi".to_string();
    let alice_keypair = generate_keypair();
    let alice_id = "alice".to_string();
    let bob_keypair = generate_keypair();
    let bob_id = "bob".to_string();

    // true block
    let mut tx_create_satoshi =
        Transaction::new(TransactionData::CreateAccount(
            satoshi_id.clone(), satoshi_keypair.public.clone()), Some(satoshi_id.clone()));
    tx_create_satoshi.signature =
        Some(satoshi_keypair.sign(tx_create_satoshi.hash().as_bytes()).to_bytes());

    let mut block = Block::new(None);
    block.add_transaction(tx_create_satoshi);
    assert!(mining(&mut block, &bc).is_ok());

    assert!(bc.append_block(block).is_ok());

    // fail block
    let mut block = Block::new(bc.get_last_block_hash());
    let mut tx_create_alice =
        Transaction::new(TransactionData::CreateAccount(
            alice_id.clone(), alice_keypair.public.clone()), Some(alice_id.clone()));

    let mut tx_create_bob =
        Transaction::new(TransactionData::CreateAccount(
            bob_id.clone(), bob_keypair.public.clone()), Some(bob_id.clone()));

    tx_create_alice.signature =
        Some(alice_keypair.sign(tx_create_alice.hash().as_bytes()).to_bytes());
    tx_create_bob.signature =
        Some(bob_keypair.sign(tx_create_bob.hash().as_bytes()).to_bytes());

    block.add_transaction(tx_create_alice);
    block.add_transaction(tx_create_bob.clone());
    block.add_transaction(tx_create_bob);
    assert!(mining(&mut block, &bc).is_ok());

    assert!(bc.append_block(block).is_err());
    assert!(bc.get_account_by_id(satoshi_id.clone()).is_some());
    assert!(bc.get_account_by_id(alice_id.clone()).is_none());
    assert!(bc.get_account_by_id(bob_id.clone()).is_none());
}

#[test]
fn test_validate_blockchain() {
    let bc = &mut Blockchain::new();

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = generate_account_id();

    let mut tx_create_account =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk), Some(user1_id.clone()));

    let tx_mint_init_supply:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: user1_id.clone(),
            amount: 100_000_000,
        },
        None,
    );

    tx_create_account.signature =
        Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());

    assert!(
        append_block_with_tx(bc, vec![tx_create_account, tx_mint_init_supply]).is_ok()
    );

    let block1 = create_block(bc, generate_account_id());
    assert!(bc.append_block(block1.clone()).is_ok());
    let block2 = common::create_block(bc, generate_account_id());
    assert!(bc.append_block(block2.clone()).is_ok());

    assert!(bc.validate().is_ok());

    let mut iter = bc.blocks.iter_mut();
    iter.next();
    iter.next();
    let block = iter.next().unwrap();
    block.transactions[1].data = TransactionData::MintInitialSupply {
        to: user1_id.clone(),
        amount: 100,
    };

    assert!(bc.validate().is_err());
}

#[test]
fn test_hash() {
    let mut block = Block::new(None);
    let user1_keypair = generate_keypair();
    let mut tx = Transaction::new(
        TransactionData::CreateAccount("alice".to_string(), user1_keypair.public.clone()),
        Some("alice".to_string())
    );
    tx.signature = Some(user1_keypair.sign(tx.hash().as_bytes()).to_bytes());

    let hash1 = block.hash();

    block.add_transaction(tx);
    block.set_nonce(1);
    let hash2 = block.hash();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_create_genesis_block() {
    let bc = &mut Blockchain::new();

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = generate_account_id();

    let mut tx_create_account =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                     Some(user1_id.clone()));

    let tx_mint_init_supply:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: user1_id.clone(),
            amount: 100_000_000,
        },
    None,
    );

    tx_create_account.signature =
        Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());

    assert!(append_block_with_tx(bc, vec![tx_create_account, tx_mint_init_supply]).is_ok());

    let satoshi = bc.get_account_by_id(user1_id.clone());

    assert!(satoshi.is_some());
    assert_eq!(satoshi.unwrap().balance, 100_000_000);
}

#[test]
fn test_create_genesis_block_fails() {
    let mut bc = Blockchain::new();

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = "satoshi".to_string();

    let mut tx_create_account =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                     Some(user1_id.clone()));

    let tx_mint_init_supply:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: user1_id.clone(),
            amount: 100_000_000,
        },
    None,
    );

    tx_create_account.signature =
        Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());

    let mut block = Block::new(None);
    block.add_transaction(tx_mint_init_supply);
    block.add_transaction(tx_create_account);
    assert!(mining(&mut block, &bc).is_ok());

    assert_eq!(
        bc.append_block(block).err().unwrap(),
        "Error during tx execution: Invalid account.".to_string()
    );
}

#[test]
fn test_account_creating() {
    let bc = &mut Blockchain::new();
    let mut block = Block::new(None);

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = generate_account_id();
    let mut tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some(user1_id.clone()));

    tx_create_account_user1.signature =
        Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());

    block.add_transaction(tx_create_account_user1.clone());
    assert!(mining(&mut block, bc).is_ok());

    assert!(bc.append_block(block.clone()).is_ok());

    let test_user = bc.get_account_by_id(user1_id.clone());
    assert!(test_user.is_some());
    assert_eq!(test_user.unwrap().public_key, user1_pk);
}

#[test]
fn test_accounts_exist() {
    let bc = &mut Blockchain::new();
    let user1_id = generate_account_id();
    let user2_id = generate_account_id();

    let block = create_block_and_tx(
        bc,vec![1000,10],90, user1_id.clone(), user2_id.clone());

    assert!(bc.append_block(block.clone()).is_ok());

    let test_user1 = bc.get_account_by_id(user1_id.clone());
    let test_user2 = bc.get_account_by_id(user2_id.clone());
    assert!(test_user1.is_some());
    assert!(test_user2.is_some());
    assert_eq!(test_user1.unwrap().balance, 910);
    assert_eq!(test_user2.unwrap().balance, 100);
}

#[test]
fn test_sender_doesnt_exist() {
    let bc = &mut Blockchain::new();
    let block = create_block(bc, "satoshi".to_string());
    assert!(bc.append_block(block.clone()).is_ok());

    let tx_transfer1 = Transaction::new(
    TransactionData::Transfer {
        to: "satoshi".to_string(),
        amount: 100,
    },
    Some("alice".to_string()),
    );

    assert!(
        append_block_with_tx(bc, vec![tx_transfer1.clone()]).is_err()
    );
}

#[test]
fn test_receiver_doesnt_exist() {
    let bc = &mut Blockchain::new();
    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;

    let mut tx_create_account =
        Transaction::new(TransactionData::CreateAccount("satoshi".to_string(), user1_pk),
                         Some("satoshi".to_string()));

    let tx_mint_init_supply:Transaction = Transaction::new(
        TransactionData::MintInitialSupply {
            to: "satoshi".to_string(),
            amount: 100_000_000,
        },
    None,
    );

    tx_create_account.signature =
        Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());

    assert!(
        append_block_with_tx(bc, vec![tx_create_account.clone(), tx_mint_init_supply.clone()],).is_ok()
    );

    let tx_transfer1 = Transaction::new(
    TransactionData::Transfer {
        to: "alice".to_string(),
        amount: 100,
    },
    Some("satoshi".to_string()),
    );

    assert!(
        append_block_with_tx(bc, vec![tx_transfer1.clone()]).is_err()
    );
 }

#[test]
fn test_not_enough_money() {
    let bc = &mut Blockchain::new();
    let user1_id = generate_account_id();
    let user2_id = generate_account_id();

    let block = create_block_and_tx(
        bc,vec![1000,10],2000, user1_id.clone(), user2_id.clone());

    assert!(bc.append_block(block.clone()).is_err());
}

#[test]
fn test_invalid_signature() {
    let bc = &mut Blockchain::new();

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = generate_account_id();
    let mut tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some(user1_id.clone()));

    assert!(
        append_block_with_tx(bc, vec![tx_create_account_user1.clone()]).is_err()
    );

    tx_create_account_user1.signature =
        Some(user1_keypair.sign("hello".as_bytes()).to_bytes());
    assert!(
        append_block_with_tx(bc, vec![tx_create_account_user1.clone()]).is_err()
    );

    tx_create_account_user1.signature =
        Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());
    assert!(
        append_block_with_tx(bc, vec![tx_create_account_user1.clone()]).is_ok()
    );
}

#[test]
fn creating_account_false() {
    let bc = &mut Blockchain::new();

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let user1_id = generate_account_id();
    let tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some("alice".to_string()));

    assert!(
        append_block_with_tx(bc, vec![tx_create_account_user1.clone()]).is_err()
    );
}