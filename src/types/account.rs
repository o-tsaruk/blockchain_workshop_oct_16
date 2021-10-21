use crate::types::{Balance, PK};


#[derive(Debug, Clone)]
pub enum AccountType {
    User,
    Contract,
}

#[derive(Debug, Clone)]
pub struct Account {
    account_type: AccountType,
    pub(crate) balance: Balance,
    pub(crate) public_key : PK,
}

impl Account {
    pub fn new(account_type: AccountType, public_key : PK) -> Self {
        Self {
            account_type,
            balance: 0,
            public_key,
        }
    }
}

