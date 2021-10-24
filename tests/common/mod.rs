use ed25519_dalek::Signer;
use blockchain_workshop::traits::Hashable;
use blockchain_workshop::types::{AccountId, Balance, Block, Blockchain, Error, Transaction, TransactionData};
use blockchain_workshop::utils::{generate_keypair, mining};

pub fn create_block(bc: &mut Blockchain, user1_id: AccountId) -> Block {
    let mut block = Block::new(bc.get_last_block_hash());

    let user1_keypair = generate_keypair();
    let user1_pk = user1_keypair.public;
    let mut tx_create_account_user1 =
        Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
                         Some(user1_id.clone()));

    tx_create_account_user1.signature =
        Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());

    block.add_transaction(tx_create_account_user1.clone());

    assert!(mining(&mut block, bc).is_ok());

    block.clone()
}

pub fn create_block_and_tx(bc: &mut Blockchain, mint_amount: Vec<Balance>, tx_amount: Balance,
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

    block.add_transaction(tx_create_account_user1.clone());
    block.add_transaction(tx_mint_init_supply_user1.clone());
    block.add_transaction(tx_create_account_user2.clone());
    block.add_transaction(tx_mint_init_supply_user2.clone());
    block.add_transaction(tx_transfer1.clone());

    assert!(mining(&mut block, bc).is_ok());

    block.clone()
}

pub fn append_block_with_tx(
    bc: &mut Blockchain,
    transactions: Vec<Transaction>,
) -> Result<(), Error> {
    let mut block = Block::new(bc.get_last_block_hash());

    for tx in transactions {
        block.add_transaction(tx);
    }

    assert!(mining(&mut block, bc).is_ok());

    bc.append_block(block)
}