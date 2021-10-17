use crate::traits::{Hashable, WorldState};
use crate::types::{AccountId, AccountType, Balance, Error, Hash, Timestamp};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};

#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub(crate) data: TransactionData,
    signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TransactionData {
    CreateAccount(AccountId),
    MintInitialSupply { to: AccountId, amount: Balance },
    Transfer { to: AccountId, amount: Balance },
}

impl Transaction {
    pub fn new(data: TransactionData, from: Option<AccountId>) -> Self {
        Self {
            nonce: 0,
            timestamp: 0,
            from,
            data,
            signature: None,
        }
    }

    pub fn execute<T: WorldState>(&self, state: &mut T, is_genesis: bool) -> Result<(), Error> {
        //TODO Task 2: Implement signature
        match &self.data {
            TransactionData::CreateAccount(account_id) => {
                state.create_account(account_id.clone(), AccountType::User)
            }
            TransactionData::MintInitialSupply { to, amount } => {
                if !is_genesis {
                    return Err("Initial supply can be minted only in genesis block.".to_string());
                }
                match state.get_account_by_id_mut(to.clone()) {
                    Some(account) => {
                        account.balance += amount;
                        println!("Start {} balance: {}", to.clone(), account.balance);
                        Ok(())
                    }
                    None => Err("Invalid account.".to_string()),
                }
            }
            // TODO Task 1: Implement transfer transition function
            // 1. Check that receiver and sender accounts exist
            // 2. Check sender balance
            // 3. Change sender/receiver balances and save to state
            // 4. Test
            TransactionData::Transfer { to, amount } => {

                // Taking Sender's &AccountId
                let sender;
                let sender_account = match &self.from {
                    Some(tmp) => {
                        sender = tmp;
                        state.get_account_by_id(tmp.clone())
                    },
                    None => {
                        return Err("Sender name doesn't exist".to_string());
                    }
                };

                // If sender account exist
                if sender_account.is_none() {
                    dbg!("Sender account doesn't exist");
                    Err("Sender account doesn't exist".to_string())
                } else {
                    // Check sender's balance
                    let sender_account = sender_account.unwrap();
                    if Transaction::is_enough(&sender_account.balance, amount) {

                        // If receiver account exist, send money
                        match state.get_account_by_id_mut(to.clone()) {
                            Some(receiver) => {
                                receiver.balance += amount;
                                println!("Receiver {} balance: {}", to.clone(), receiver.balance);
                                let acc = (state.get_account_by_id_mut(sender.clone())).unwrap();
                                acc.balance -= amount;
                                println!("Sender {} balance: {}", sender.clone(), acc.balance);
                                return Ok(());
                            },
                            None => {
                                dbg!("Receiver account doesn't exist");
                                return Err("Receiver doesn't exist".to_string())
                            }

                        }
                    } else {
                        println!("Sender haven't enough money!");
                        return Err("Sender haven't enough money!".to_string());
                    }
                }
            }

        }
    }

    // Chek sender's balance
    fn is_enough(acc : &Balance, amount: &Balance) -> bool {
        if acc > amount {
            true
        } else {
            false
        }
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();

        hasher.update(format!(
            "{:?}",
            (
                self.nonce,
                self.timestamp,
                self.from.clone(),
                self.data.clone()
            )
        ));

        hex::encode(hasher.finalize_fixed())
    }
}


#[cfg(test)]
mod tests {
    use crate::types::{Block, Blockchain, Transaction, TransactionData};
    use crate::utils::append_block_with_tx;

    #[test]
    fn test_accounts_exist() {
        let bc = &mut Blockchain::new();

        let tx_create_account_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);

        let tx_mint_init_supply_alice:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "alice".to_string(),
                amount: 250,
            },
            None,
        );

        let tx_create_account_sasha =
            Transaction::new(TransactionData::CreateAccount("sasha".to_string()), None);

        let tx_mint_init_supply_sasha:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "sasha".to_string(),
                amount: 1250,
            },
            None,
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 100,
        },
        Some("sasha".to_string()),
        );

        append_block_with_tx(bc, 1, vec![tx_create_account_alice,
                                         tx_mint_init_supply_alice,tx_create_account_sasha,
                                        tx_mint_init_supply_sasha]);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1]).is_ok()
        )
    }

    #[test]
    fn test_sender_doesnt_exist() {
        let bc = &mut Blockchain::new();

        let tx_create_account_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);

        let tx_mint_init_supply_alice:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "alice".to_string(),
                amount: 250,
            },
            None,
        );

        let tx_create_account_sasha =
            Transaction::new(TransactionData::CreateAccount("sasha".to_string()), None);

        let tx_mint_init_supply_sasha:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "sasha".to_string(),
                amount: 1250,
            },
            None,
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 100,
        },
        Some("sasha".to_string()),
        );

        append_block_with_tx(bc, 1, vec![tx_create_account_alice,
                                         tx_mint_init_supply_alice]);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1.clone()]).is_err()
        );
    }

    #[test]
    fn test_receiver_doesnt_exist() {
        let bc = &mut Blockchain::new();

        let tx_create_account_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);

        let tx_mint_init_supply_alice:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "alice".to_string(),
                amount: 250,
            },
            None,
        );

        let tx_create_account_sasha =
            Transaction::new(TransactionData::CreateAccount("sasha".to_string()), None);

        let tx_mint_init_supply_sasha:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "sasha".to_string(),
                amount: 1250,
            },
            None,
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 100,
        },
        Some("sasha".to_string()),
        );

        append_block_with_tx(bc, 1, vec![tx_create_account_sasha,
                                         tx_mint_init_supply_sasha]);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1.clone()]).is_err()
        );
    }

    #[test]
    fn test_not_enough_money() {
        let bc = &mut Blockchain::new();

        let tx_create_account_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);

        let tx_mint_init_supply_alice:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "alice".to_string(),
                amount: 250,
            },
            None,
        );

        let tx_create_account_sasha =
            Transaction::new(TransactionData::CreateAccount("sasha".to_string()), None);

        let tx_mint_init_supply_sasha:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "sasha".to_string(),
                amount: 1250,
            },
            None,
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 1500,
        },
        Some("sasha".to_string()),
        );

        append_block_with_tx(bc, 1, vec![tx_create_account_alice,
                                         tx_mint_init_supply_alice, tx_create_account_sasha,
                                         tx_mint_init_supply_sasha]);

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1.clone()]).is_err()
        );
    }

    #[test]
    fn test_creation() {
        let mut block = Block::new(None);

        let tx_sasha =
            Transaction::new(TransactionData::CreateAccount("sasha".to_string()), None);

        let tx_mint_init_supply_sasha:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "sasha".to_string(),
                amount: 5000,
            },
            None,
        );

        let tx_alice =
            Transaction::new(TransactionData::CreateAccount("alice".to_string()), None);

        let tx_mint_init_supply_alice:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "alice".to_string(),
                amount: 100,
            },
            None,
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 1000,
        },
        Some("sasha".to_string()),
        );

        block.set_nonce(1);
        block.add_transaction(tx_sasha);
        block.add_transaction(tx_mint_init_supply_sasha);
        block.add_transaction(tx_alice);
        block.add_transaction(tx_mint_init_supply_alice);
        block.add_transaction(tx_transfer1);

        dbg!(block);

    }
}
