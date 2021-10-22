use crate::traits::{Hashable, WorldState};
use crate::types::{Account, AccountId, AccountType, Block, Chain, Error, EXPECTED_TIME, Hash, MAX_TARGET, PK, Target, Timestamp, Transaction};
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use crate::utils::check_target;

#[derive(Default, Debug)]
pub struct Blockchain {
    blocks: Chain<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
    pub(crate) current_target: Target,
    last_timestamp: Timestamp,
}

impl WorldState for Blockchain {
    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
        public_key: PK,
    ) -> Result<(), Error> {
        match self.accounts.entry(account_id.clone()) {
            Entry::Occupied(_) => Err(format!("AccountId already exist: {}", account_id)),
            Entry::Vacant(v) => {
                v.insert(Account::new(account_type, public_key));
                Ok(())
            }
        }
    }

    fn get_account_by_id(&self, account_id: AccountId) -> Option<&Account> {
        self.accounts.get(&account_id)
    }

    fn get_account_by_id_mut(&mut self, account_id: AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(&account_id)
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let mut bc = Blockchain {
            ..Default::default()
        };
        bc.current_target = MAX_TARGET;

        bc
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
        //TODO Task 3: Implement mining
        if !block.verify() {
            return Err("Block has invalid hash".to_string());
        }
        if check_target(self.current_target.clone(), block.hash.clone().unwrap()) == false {
            return Err("Block hash > current target!".to_string());
        }
        let is_genesis = self.blocks.len() == 0;

        if block.transactions.len() == 0 {
            return Err("Block has 0 transactions.".to_string());
        }

        let account_backup = self.accounts.clone();
        for tx in &block.transactions {
            let res = tx.execute(self, is_genesis);
            if let Err(error) = res {
                self.accounts = account_backup;
                return Err(format!("Error during tx execution: {}", error));
            }
        }

        // TODO Task 3: Append block only if block.hash < target
        if !is_genesis {
            let actual = block.timestamp.clone() - self.last_timestamp.clone();
            let mut ratio: f64 = (actual as f64)/EXPECTED_TIME;
            if ratio < 0.25 {
                ratio = 0.25;
            } else if ratio > 4.0 {
                ratio = 4.0;
            }

            let new_target: f64 = (self.current_target as f64) * ratio;
            if new_target.ceil() >= (MAX_TARGET as f64) {
                self.current_target = MAX_TARGET;
            } else {
                self.current_target = new_target.ceil() as u64;
            }
        }

        self.last_timestamp = block.timestamp;
        self.blocks.append(block);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut block_num = self.blocks.len();
        let mut prev_block_hash: Option<Hash> = None;

        for block in self.blocks.iter() {
            let is_genesis = block_num == 1;

            if !block.verify() {
                return Err(format!("Block {} has invalid hash", block_num));
            }

            if !is_genesis && block.prev_hash.is_none() {
                return Err(format!("Block {} doesn't have prev_hash", block_num));
            }

            if is_genesis && block.prev_hash.is_some() {
                return Err("Genesis block shouldn't have prev_hash".to_string());
            }

            if block_num != self.blocks.len() {
                if let Some(prev_block_hash) = &prev_block_hash {
                    if prev_block_hash != &block.hash.clone().unwrap() {
                        return Err(format!(
                            "Block {} prev_hash doesn't match Block {} hash",
                            block_num + 1,
                            block_num
                        ));
                    }
                }
            }

            prev_block_hash = block.prev_hash.clone();
            block_num -= 1;
        }

        Ok(())
    }

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|block| block.hash())
    }

}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Signer;
    use super::*;
    use crate::types::{TransactionData};
    use crate::utils::{append_block_with_tx, create_block, create_block_and_tx, generate_account_id, generate_keypair, mining};

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }

    // Probability of long-term execution
    // #[test]
    // fn test_create() {
    //     let bc = &mut Blockchain::new();
    //
    //     let mut block1 = create_block_and_tx(bc, vec![10,0], 5,
    //                                      "bob".to_string(), "alice".to_string());
    //
    //     assert!(bc.append_block(block1.clone()).is_ok());
    //
    //     let mut block2 = create_block(bc,generate_account_id());
    //     assert!(bc.append_block(block2.clone()).is_ok());
    //
    //     let mut block3 = create_block(bc,generate_account_id());
    //     assert!(bc.append_block(block3.clone()).is_ok());
    //
    //     assert_eq!(bc.get_last_block_hash(), block3.hash.clone());
    //     assert!(bc.validate().is_ok());
    //     dbg!(block1.hash);
    //     dbg!(block2.hash);
    //     dbg!(block3.hash);
    // }

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

    // Probability of long-term execution
    // #[test]
    // fn test_validate() {
    //     let bc = &mut Blockchain::new();
    //
    //     let user1_keypair = generate_keypair();
    //     let user1_pk = user1_keypair.public;
    //     let user1_id = generate_account_id();
    //
    //     let mut tx_create_account =
    //         Transaction::new(TransactionData::CreateAccount(user1_id.clone(), user1_pk),
    //                      Some(user1_id.clone()));
    //
    //     let tx_mint_init_supply:Transaction = Transaction::new(
    //         TransactionData::MintInitialSupply {
    //             to: user1_id.clone(),
    //             amount: 100_000_000,
    //         },
    //     None,
    //     );
    //
    //     tx_create_account.signature =
    //         Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());
    //
    //     assert!(
    //         append_block_with_tx(bc, vec![tx_create_account, tx_mint_init_supply]).is_ok()
    //     );
    //
    //     let block1 = create_block(bc, generate_account_id());
    //     assert!(bc.append_block(block1.clone()).is_ok());
    //     let block2 = create_block(bc, generate_account_id());
    //     assert!(bc.append_block(block2.clone()).is_ok());
    //
    //     assert!(bc.validate().is_ok());
    //
    //     let mut iter = bc.blocks.iter_mut();
    //     iter.next();
    //     iter.next();
    //     let block = iter.next().unwrap();
    //     block.transactions[1].data = TransactionData::MintInitialSupply {
    //         to: user1_id.clone(),
    //         amount: 100,
    //     };
    //
    //     assert!(bc.validate().is_err());
    // }
}
