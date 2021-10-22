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
pub type Timestamp = u64;
pub type AccountId = String;
pub type Balance = u128;
pub type PK = PublicKey;
pub type SignatureBytes = [u8; 64];
pub type Error = String;
pub type Target = u64;

// for first block
// 0x00fffff000000000000000000000000000000000000000000000000000000000 => 0x1ffffff0
pub const MAX_TARGET: Target = 536_870_896;
pub const EXPECTED_TIME: f64 = 1.5;
