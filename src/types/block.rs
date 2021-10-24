use crate::traits::Hashable;
use crate::types::{Hash, Timestamp, Transaction};
use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};
use crate::utils::generate_timestamp;

#[derive(Default, Debug, Clone)]
pub struct Block {
    nonce: u128,
    pub(crate) timestamp: Timestamp,
    pub hash: Option<Hash>,
    pub(crate) prev_hash: Option<Hash>,
    pub transactions: Vec<Transaction>,
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
