use crate::traits::Hashable;
use crate::types::{Hash, Timestamp, Transaction};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use crate::utils::generate_timestamp;

#[derive(Default, Debug, Clone)]
pub struct Block {
    nonce: u128,
    pub(crate) timestamp: Timestamp,
    pub(crate) hash: Option<Hash>,
    pub(crate) prev_hash: Option<Hash>,
    pub(crate) transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(prev_hash: Option<Hash>) -> Self {
        let mut block = Block {
            prev_hash,
            ..Default::default()
        };
        block.timestamp = generate_timestamp();
        block.update_hash();

        block
    }

    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.update_hash();
    }

    pub fn verify(&self) -> bool {
        matches!(&self.hash, Some(hash) if hash == &self.hash())
    }

    fn update_hash(&mut self) {
        self.hash = Some(self.hash());
    }
}

impl Hashable for Block {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();
        hasher.update(format!("{:?}", (self.prev_hash.clone(), self.nonce)).as_bytes());
        for tx in self.transactions.iter() {
            hasher.update(tx.hash())
        }

        hex::encode(hasher.finalize_fixed())
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Signer;
    use super::*;
    use crate::types::{Blockchain, TransactionData};
    use crate::utils::{generate_keypair, mining};

    #[test]
    fn test_creation() {
        let bc = &mut Blockchain::new();
        let mut block = Block::new(None);
        let user1_keypair = generate_keypair();
        let mut tx = Transaction::new(
            TransactionData::CreateAccount("alice".to_string(), user1_keypair.public.clone()),
            Some("alice".to_string())
        );

        tx.signature = Some(user1_keypair.sign(tx.hash().as_bytes()).to_bytes());
        block.add_transaction(tx);

        assert!(mining(&mut block, bc).is_ok());

        dbg!(block.clone());
        assert!(bc.append_block(block.clone()).is_ok());
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
}
