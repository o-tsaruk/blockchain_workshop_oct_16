use crate::traits::{Hashable, WorldState};
use crate::types::{AccountId, AccountType, Balance, Error, Hash, PK, SignatureBytes, Timestamp};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use ed25519_dalek::{Signature, Verifier};


#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    from: Option<AccountId>,
    pub data: TransactionData,
    pub signature: Option<SignatureBytes>,
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

        match &self.data {

            TransactionData::CreateAccount(account_id, pub_key) => {
                Transaction::create_account(&self, state, account_id, pub_key)
            }

            TransactionData::MintInitialSupply { to, amount } => {
                Transaction::mint_init_supply(&self, state, to, amount, is_genesis)
            }

            TransactionData::Transfer { to, amount } => {
                Transaction::transfer(&self, state, to, amount)
            }
        }
    }

    fn create_account<T: WorldState>(&self, state: &mut T, account_id: &AccountId, pub_key: &PK) -> Result<(), Error> {
        if self.from.is_none() {
            return Err("Sender name doesn't exist!".to_string());
        }

        let sender_id = self.from.clone().unwrap();
        let sender_acc = state.get_account_by_id(sender_id.clone());

        // if sender account is created by itself
        // or sender account already exist: verify signature
        if (sender_acc.is_none()) && (&sender_id != account_id) {
            return Err("Creating account by other non-existent account!".to_string());
        }

        let res = Transaction::check_tx_create_sign(self, *pub_key, self.signature.clone());
        if let Err(error) = res {
            return Err(format!("Error during tx execution: {}", error));
        }

        state.create_account(account_id.clone(), AccountType::User, *pub_key)
    }

    fn mint_init_supply<T: WorldState>(&self, state: &mut T, to: &AccountId, amount: &Balance, is_genesis: bool) -> Result<(), Error>
    {
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

    fn transfer<T: WorldState>(&self, state: &mut T, to: &AccountId, amount: &Balance) -> Result<(), Error> {
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
        } else if self.signature.is_none() {
            return Err("Signature doesn't exist!".to_string());
        }

        // If signature is true
        let sender_account = sender_account.unwrap();
        let signature_presence = Transaction::check_tx_sign(
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

    // Chek sender's balance
    fn is_enough(acc : &Balance, amount: &Balance) -> bool {
        if acc >= amount { return true; }

        false
    }

    fn check_tx_create_sign(&self, pub_key: PK, signature: Option<SignatureBytes>) -> Result<(), Error> {
        if signature.is_none() {
            return Err("Signature doesn't exist!".to_string());
        }

        let verification = pub_key
            .verify(self.hash().as_bytes(), &Signature::from(signature.unwrap())).is_ok();

        if verification {
            return Ok(());
        }

        return Err("Verify signature error!".to_string())
    }

    fn check_tx_sign(&self, pub_key: PK, signature: Option<SignatureBytes>) -> bool {
        return pub_key
            .verify(self.hash().as_bytes(), &Signature::from(signature.unwrap())).is_ok()
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
