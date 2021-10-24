use crate::traits::{Hashable, WorldState};
use crate::types::{Account, AccountId, AccountType, Block, Chain, COEFFICIENT_LENGTH, Error, EXPECTED_TIME, Hash, MAX_COMPACT_FORM, MAX_TARGET, PK, Target, Timestamp, Transaction};
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use crate::utils::check_target;

#[derive(Default, Debug)]
pub struct Blockchain {
    pub blocks: Chain<Block>,
    accounts: HashMap<AccountId, Account>,
    transaction_pool: Vec<Transaction>,
    last_timestamp: Timestamp,
    pub(crate) current_target: Target,
    pub(crate) compact_form: String,
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
        bc.compact_form = MAX_COMPACT_FORM.to_string();

        bc
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
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

        if !is_genesis {
            Blockchain::target_adjust(self, block.timestamp.clone());
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

    fn target_adjust(&mut self, block_timestamp: Timestamp) {
        let actual = block_timestamp - self.last_timestamp.clone();
        let mut ratio: f64 = (actual as f64)/EXPECTED_TIME;
        if ratio < 0.25 {
            ratio = 0.25;
        } else if ratio > 4.0 {
            ratio = 4.0;
        }

        let start_exp = &self.compact_form[..2];      // exponent
        let start_coef  = &self.compact_form[2..];    // coefficient
        let dec_exp = u64::from_str_radix(start_exp, 16);
        let dec_coef = u64::from_str_radix(start_coef, 16);
        let dec_new_coef: f64 = (dec_coef.unwrap() as f64) * ratio;

        let new_compact_form =
            Blockchain::check_target_adjust(dec_exp.unwrap(), dec_new_coef, start_coef, ratio);
        let new_target = u64::from_str_radix(&new_compact_form, 16);

        if new_target.clone().unwrap() >= MAX_TARGET {
            self.current_target = MAX_TARGET;
            self.compact_form = MAX_COMPACT_FORM.to_string();
        } else {
            self.current_target = new_target.unwrap();
            self.compact_form = new_compact_form;
        }
    }

    fn check_target_adjust(dec_exp: u64, mut dec_new_coef: f64, start_coef: &str, ratio: f64) -> String {
        if (start_coef.chars().nth(2).unwrap() == '0') && (ratio < 1.0)  {
            dec_new_coef *= 16.0;
        }

        let mut hex_new_coef = format!("{:x}", dec_new_coef.clone().ceil() as i64);
        let mut dec_new_exp: u64 = dec_exp;
        if hex_new_coef.len() == COEFFICIENT_LENGTH-1 {
            hex_new_coef += "0";
            dec_new_exp -= 1;
        }
        else if hex_new_coef.len() > COEFFICIENT_LENGTH {
            hex_new_coef = "0".to_owned() + &hex_new_coef[..COEFFICIENT_LENGTH-1].to_string();
            dec_new_exp += 1;
        }

        hex::encode(vec![dec_new_exp as u8]) + &hex_new_coef
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_last_block_hash(), None);
    }
}
