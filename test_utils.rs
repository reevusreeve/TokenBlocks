// utils/test_utils.rs

use near_sdk::test_utils::{VMContextBuilder, accounts};
use near_sdk::{testing_env, AccountId, Balance, VMContext};
use crate::*;

pub struct TestUtils;

impl TestUtils {
    pub fn get_context(
        predecessor: AccountId,
        deposit: Balance,
        timestamp: u64
    ) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(AccountId::new_unchecked("contract.near".to_string()))
            .predecessor_account_id(predecessor)
            .attached_deposit(deposit)
            .block_timestamp(timestamp)
            .is_view(false);
        builder
    }

    pub fn get_accounts() -> [AccountId; 4] {
        [
            AccountId::new_unchecked("owner.near".to_string()),
            AccountId::new_unchecked("alice.near".to_string()),
            AccountId::new_unchecked("bob.near".to_string()),
            AccountId::new_unchecked("carol.near".to_string()),
        ]
    }

    pub fn setup_contract() -> (AccountId, TokenBlocks) {
        let owner_id = AccountId::new_unchecked("owner.near".to_string());
        let context = TestUtils::get_context(
            owner_id.clone(),
            0,
            0
        );
        testing_env!(context.build());
        
        let contract = TokenBlocks::new(
            owner_id.clone(),
            None,
            None,
            None
        );
        
        (owner_id, contract)
    }

    pub fn to_yocto(near_amount: u128) -> Balance {
        near_amount * 10u128.pow(24)
    }

    pub fn assert_token_balance(
        contract: &TokenBlocks,
        account_id: &AccountId,
        token_id: TokenId,
        expected: Balance
    ) {
        let balance = contract.get_token_balance(account_id, token_id);
        assert_eq!(balance, expected, "Incorrect token balance");
    }

    pub fn create_test_token(
        contract: &mut TokenBlocks,
        creator: AccountId,
        deposit: Balance
    ) -> TokenId {
        let context = TestUtils::get_context(creator.clone(), deposit, 0);
        testing_env!(context.build());

        let metadata = TokenMetadata {
            title: "Test Token".to_string(),
            description: Some("Test Description".to_string()),
            media: None,
            media_hash: None,
            copies: Some(1000),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            extra: None,
        };

        contract.create_token("ipfs://test".to_string(), metadata)
    }

    pub fn advance_time(seconds: u64) {
        let mut context = VMContextBuilder::new();
        let current = env::block_timestamp();
        context.block_timestamp(current + seconds * 1_000_000_000);
        testing_env!(context.build());
    }

    pub fn assert_expected_events(expected_events: Vec<&str>) {
        let events = env::mock_all_events();
        assert_eq!(
            events.len(),
            expected_events.len(),
            "Expected {} events, but got {}",
            expected_events.len(),
            events.len()
        );
        for (i, event) in events.iter().enumerate() {
            assert!(
                event.contains(expected_events[i]),
                "Event {} does not match expected content",
                i
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_setup() {
        let (owner_id, contract) = TestUtils::setup_contract();
        assert_eq!(contract.owner_id, owner_id);
    }

    #[test]
    fn test_token_creation() {
        let (owner_id, mut contract) = TestUtils::setup_contract();
        let deposit = TestUtils::to_yocto(1);
        let token_id = TestUtils::create_test_token(
            &mut contract,
            owner_id.clone(),
            deposit
        );
        assert_eq!(token_id, 0);
    }

    #[test]
    fn test_time_advancement() {
        let start_time = env::block_timestamp();
        TestUtils::advance_time(100);
        assert_eq!(
            env::block_timestamp() - start_time,
            100 * 1_000_000_000
        );
    }
}
