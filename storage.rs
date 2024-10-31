// utils/storage.rs

use near_sdk::borsh::{self, BorshSerialize};
use near_sdk::{env, Balance, Promise};

pub struct Storage;

impl Storage {
    // Constants
    pub const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;
    pub const MINIMUM_STORAGE_BALANCE: Balance = 1_000_000_000_000_000_000_000;

    pub fn get_storage_usage<T: BorshSerialize>(data: &T) -> Balance {
        data.try_to_vec()
            .expect("Failed to serialize data")
            .len() as Balance
    }

    pub fn assert_storage_covered(storage_used: Balance, attached_deposit: Balance) {
        let required_cost = storage_used * Self::STORAGE_PRICE_PER_BYTE;
        assert!(
            attached_deposit >= required_cost,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost
        );
    }

    pub fn refund_storage(initial_storage: u64, attached_deposit: Balance) {
        let current_storage = env::storage_usage();
        let storage_used = current_storage - initial_storage;
        let storage_cost = storage_used as Balance * Self::STORAGE_PRICE_PER_BYTE;
        
        if attached_deposit > storage_cost {
            Promise::new(env::predecessor_account_id())
                .transfer(attached_deposit - storage_cost);
        }
    }

    pub fn calculate_required_storage<T: BorshSerialize>(
        data: &T,
        extra_bytes: u64
    ) -> Balance {
        let size = Self::get_storage_usage(data) + extra_bytes;
        size * Self::STORAGE_PRICE_PER_BYTE
    }

    pub fn assert_minimum_storage(account_balance: Balance) {
        assert!(
            account_balance >= Self::MINIMUM_STORAGE_BALANCE,
            "Account must maintain minimum storage balance"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    fn test_storage_calculations() {
        let test_data = "test data".to_string();
        let storage_usage = Storage::get_storage_usage(&test_data);
        assert!(storage_usage > 0);
        
        let required_deposit = storage_usage * Storage::STORAGE_PRICE_PER_BYTE;
        Storage::assert_storage_covered(storage_usage, required_deposit);
    }

    #[test]
    fn test_storage_refund() {
        let initial_storage = 100;
        let deposit = Storage::STORAGE_PRICE_PER_BYTE * 200;
        // Test refund calculation
        let current_usage = 150;
        let expected_refund = deposit - 
            (current_usage - initial_storage) as Balance * 
            Storage::STORAGE_PRICE_PER_BYTE;
        assert!(expected_refund > 0);
    }
}
