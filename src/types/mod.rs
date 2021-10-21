mod account;
mod block;
mod blockchain;
mod chain;
mod transaction;

pub use ed25519_dalek::PublicKey;
pub use account::{Account, AccountType};
pub use block::Block;
pub use blockchain::Blockchain;
pub use chain::Chain;
pub use transaction::{Transaction, TransactionData};

pub type Hash = String;
pub type Timestamp = u128;
pub type AccountId = String;
pub type Balance = u128;
pub type PK = PublicKey;
pub type SignatureBytes = [u8; 64];
pub type Error = String;
