// utils/validation.rs

use near_sdk::{env, AccountId, Balance};
use crate::*;

pub struct Validation;

impl Validation {
    // Token Metadata Validation
    pub fn assert_valid_metadata(metadata: &TokenMetadata) -> bool {
        // Title validation
        assert!(!metadata.title.is_empty(), "Title cannot be empty");
        assert!(metadata.title.len() <= 100, "Title too long");

        // Description validation
        if let Some(ref desc) = metadata.description {
            assert!(desc.len() <= 1000, "Description too long");
        }

        // Media validation
        if let Some(ref media) = metadata.media {
            assert!(!media.is_empty(), "Media hash cannot be empty");
            assert!(
                media.starts_with("ipfs://") || media.starts_with("ar://"),
                "Invalid media protocol"
            );
        }

        // Copies validation
        if let Some(copies) = metadata.copies {
            assert!(copies > 0, "Copies must be greater than 0");
            assert!(copies <= 1_000_000_000, "Too many copies");
        }

        true
    }

    // Stake Validation
    pub fn assert_valid_stake(
        amount: Balance,
        min_stake: Balance,
        user_balance: Balance
    ) -> bool {
        assert!(amount >= min_stake, "Stake amount below minimum");
        assert!(amount <= user_balance, "Insufficient balance");
        true
    }

    // Purchase Validation
    pub fn assert_valid_purchase(
        amount: Balance,
        available: Balance,
        price: Balance,
        payment: Balance
    ) -> bool {
        assert!(amount > 0, "Purchase amount must be greater than 0");
        assert!(amount <= available, "Insufficient tokens available");
        assert!(payment >= price, "Insufficient payment");
        true
    }

    // Block Phase Validation
    pub fn assert_valid_block_phase(
        block: &Block,
        current_time: u64,
        phase: BlockPhase
    ) -> bool {
        assert!(block.phase == phase, "Invalid block phase");
        match phase {
            BlockPhase::Voting => {
                assert!(current_time < block.end_time, "Voting period ended");
            }
            BlockPhase::PriorityPurchase => {
                assert!(
                    current_time >= block.end_time && 
                    current_time < block.end_time + 120_000_000_000,
                    "Not in priority purchase period"
                );
            }
            BlockPhase::PublicPurchase => {
                assert!(
                    current_time >= block.end_time + 120_000_000_000 &&
                    current_time < block.end_time + 300_000_000_000,
                    "Not in public purchase period"
                );
            }
            _ => panic!("Invalid phase check")
        }
        true
    }

    // User Access Validation
    pub fn assert_owner(
        caller: &AccountId,
        owner: &AccountId
    ) {
        assert!(
            caller == owner,
            "Only contract owner can perform this action"
        );
    }

    pub fn assert_voter_access(
        voter: &AccountId,
        stakes: &LookupMap<AccountId, StakeInfo>
    ) -> bool {
        assert!(
            stakes.get(voter).is_some(),
            "Only voters can access during priority period"
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    fn test_metadata_validation() {
        let valid_metadata = TokenMetadata {
            title: "Test Token".to_string(),
            description: Some("Valid description".to_string()),
            media: Some("ipfs://hash".to_string()),
            media_hash: None,
            copies: Some(1000),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            extra: None,
        };
        assert!(Validation::assert_valid_metadata(&valid_metadata));
    }

    #[test]
    #[should_panic(expected = "Title cannot be empty")]
    fn test_invalid_metadata() {
        let invalid_metadata = TokenMetadata {
            title: "".to_string(),
            description: None,
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            extra: None,
        };
        Validation::assert_valid_metadata(&invalid_metadata);
    }

    #[test]
    fn test_stake_validation() {
        assert!(Validation::assert_valid_stake(100, 10, 1000));
    }

    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn test_invalid_stake() {
        Validation::assert_valid_stake(1000, 10, 100);
    }
}
