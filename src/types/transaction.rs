use crate::traits::{Hashable, WorldState};
use crate::types::{Account, AccountId, AccountType, Balance, Error, Hash, PK, SignatureBytes, Timestamp};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use ed25519_dalek::{Signature, Verifier};


#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub(crate) data: TransactionData,
    pub(crate) signature: Option<SignatureBytes>,
}

#[derive(Debug, Clone)]
pub enum TransactionData {
    CreateAccount(AccountId, PK),
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

            TransactionData::CreateAccount(account_id, pub_key) => {
                if self.from.is_none() {
                    return Err("Sender name doesn't exist!".to_string());
                } else if self.signature.is_none() {
                    return Err("Signature doesn't exist!".to_string());
                }
                let sender_id = self.from.clone().unwrap();
                let sender_acc = state.get_account_by_id(sender_id.clone());

                if Transaction::check_tx_create_sign(
                    self,sender_id.clone(), account_id, pub_key, sender_acc, self.signature.clone()) {

                    state.create_account(account_id.clone(), AccountType::User, *pub_key)
                } else { return  Err("Verify signature error!".to_string());}
            }

            TransactionData::MintInitialSupply { to, amount } => {
                if !is_genesis {
                    return Err("Initial supply can be minted only in genesis block.".to_string());
                }
                match state.get_account_by_id_mut(to.clone()) {
                    Some(account) => {
                        account.balance += amount;
                        Ok(())
                    }
                    None => Err("Invalid account.".to_string()),
                }
            }

            // TODO Task 1: Implement transfer transition function
            TransactionData::Transfer { to, amount } => {

                // Taking Sender's &AccountId
                let sender;
                let sender_account = match &self.from {
                    Some(tmp) => {
                        sender = tmp;
                        state.get_account_by_id(tmp.clone())
                    },
                    None => { return Err("Sender name doesn't exist".to_string()); }
                };

                // If sender account exist
                if sender_account.is_none() {
                    return Err("Sender account doesn't exist".to_string())
                }

                // If signature is true
                let sender_account = sender_account.unwrap();
                let signature_presence = Transaction::check_tx_transfer_sign(
                        &self, sender_account.public_key.clone(), self.signature.clone());

                if signature_presence == false {
                    return  Err("Verify signature error!".to_string());
                }

                // Check sender's balance
                if Transaction::is_enough(&sender_account.balance, amount) {
                    // If receiver account exist, send money
                    match state.get_account_by_id_mut(to.clone()) {
                        Some(receiver) => {
                            receiver.balance += amount;
                            let sender_account = (state.get_account_by_id_mut(sender.clone())).unwrap();
                            sender_account.balance -= amount;
                            return Ok(());
                        },
                        None => { return Err("Receiver doesn't exist".to_string()) }
                    }
                } else { return Err("Sender haven't enough money!".to_string()); }

            }

        }
    }

    // Chek sender's balance
    fn is_enough(acc : &Balance, amount: &Balance) -> bool {
        if acc >= amount { return true; }

        false
    }

    fn check_tx_create_sign(&self, sender_id: AccountId, receiver_id: &AccountId, pub_key: &PK,
                            sender_acc: Option<&Account>, signature: Option<SignatureBytes>) -> bool {

        // if sender account is created by itself
        // or sender account already exist: verify signature
        if (sender_acc.is_none() && (&sender_id == receiver_id)) ||
            sender_acc.is_some() || signature.is_none() {

            return pub_key
                    .verify(self.hash().as_bytes(),
                            &Signature::from(signature.unwrap())).is_ok()
        }

        false
    }

    fn check_tx_transfer_sign(&self, pub_key: PK, signature: Option<SignatureBytes>) -> bool {
        if signature.is_some() {
            return pub_key
                        .verify(self.hash().as_bytes(),
                    &Signature::from(signature.unwrap())).is_ok()
        }

        false
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
    use ed25519_dalek::{Signer};
    use crate::traits::{Hashable, WorldState};
    use crate::types::{Block, Blockchain, Transaction, TransactionData};
    use crate::utils::{append_block_with_tx, create_block, create_block_and_tx, generate_account_id, generate_keypair};

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

        block.set_nonce(1);
        block.add_transaction(tx_create_account_user1.clone());
        assert!(bc.append_block(block.clone()).is_ok());

        let test_user = bc.get_account_by_id(user1_id.clone());
        assert!(test_user.is_some());
        assert_eq!(test_user.unwrap().public_key, user1_pk);
        dbg!(block.clone());
    }

    #[test]
    fn test_accounts_exist() {
        let bc = &mut Blockchain::new();
        let user1_id = generate_account_id();
        let user2_id = generate_account_id();

        let block= create_block_and_tx(
            bc,1,vec![1000,10],90, user1_id.clone(), user2_id.clone());

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
        let block = create_block(bc, 1, "satoshi".to_string());
        assert!(bc.append_block(block.clone()).is_ok());

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "satoshi".to_string(),
            amount: 100,
        },
        Some("alice".to_string()),
        );

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1.clone()]).is_err()
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

        let mut tx_mint_init_supply:Transaction = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
        None,
        );

        tx_create_account.signature =
            Some(user1_keypair.sign(tx_create_account.hash().as_bytes()).to_bytes());
        tx_mint_init_supply.signature =
            Some(user1_keypair.sign(tx_mint_init_supply.hash().as_bytes()).to_bytes());


        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account.clone()]).is_ok()
        );

        let tx_transfer1 = Transaction::new(
        TransactionData::Transfer {
            to: "alice".to_string(),
            amount: 100,
        },
        Some("satoshi".to_string()),
        );

        assert!(
            append_block_with_tx(bc, 1, vec![tx_transfer1.clone()]).is_err()
        );
     }

    #[test]
    fn test_not_enough_money() {
        let bc = &mut Blockchain::new();
        let user1_id = generate_account_id();
        let user2_id = generate_account_id();

        let block= create_block_and_tx(
            bc,1,vec![1000,10],2000, user1_id.clone(), user2_id.clone());

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
            append_block_with_tx(bc, 1, vec![tx_create_account_user1.clone()]).is_err()
        );

        tx_create_account_user1.signature =
            Some(user1_keypair.sign("hello".as_bytes()).to_bytes());
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account_user1.clone()]).is_err()
        );

        tx_create_account_user1.signature =
            Some(user1_keypair.sign(tx_create_account_user1.hash().as_bytes()).to_bytes());
        assert!(
            append_block_with_tx(bc, 1, vec![tx_create_account_user1.clone()]).is_ok()
        );
    }

}
