use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{testing_env, Balance};
//use near_sdk::json_types::U128;
use token_blocks::*;
use near_sdk::MockedBlockchain;
use token_blocks::TokenMetadata;
use near_sdk::json_types::ValidAccountId;

fn setup_voting_context(predecessor: &str, deposit: Balance) -> VMContextBuilder {
    let mut context = VMContextBuilder::new();
    context
        .predecessor_account_id(ValidAccountId::try_from(predecessor).unwrap())
        .current_account_id(ValidAccountId::try_from("contract.near").unwrap())
        .attached_deposit(deposit)
        .block_timestamp(0);
    context
}

#[test]
fn test_voting() {
    let mut context = setup_voting_context("owner.near", 0);
    testing_env!(context.build());

    let mut contract = TokenBlocks::new("owner.near".to_string());

    // Create and start a block first
    let token_id = contract.create_token(create_test_metadata());
    contract.start_block();
    
    // Verify block is active
    assert!(contract.get_current_block().is_some());
    
    // Advance time to voting phase and update phase
    context.block_timestamp(ACCEPTING_TOKENS_DURATION + 1);
    testing_env!(context.build());
    contract.update_block_phase();
    
    // Add this line to ensure phase transition
    assert_eq!(contract.get_current_block().unwrap().phase, BlockPhase::Voting);

    // Vote on token with sufficient stake
    context = setup_voting_context("voter.near", 10_000_000_000_000_000_000_000); // 10 NEAR
    testing_env!(context.build());
    
    let vote_result = contract.vote(token_id);
    assert!(vote_result);
}

#[test]
#[should_panic(expected = "Stake too low")]
fn test_vote_with_low_stake() {
    let mut context = setup_voting_context("owner.near", 0);
    testing_env!(context.build());

    let mut contract = TokenBlocks::new("owner.near".to_string());
    
    // Create and start a block first
    let token_id = contract.create_token(create_test_metadata());
    contract.start_block();
    
    // Advance time to voting phase and update phase
    context.block_timestamp(ACCEPTING_TOKENS_DURATION + 1);
    testing_env!(context.build());
    contract.update_block_phase();
    
    // Try to vote with insufficient stake
    context = setup_voting_context("voter.near", 1); // Very low stake
    testing_env!(context.build());
    contract.vote(token_id);
}

// Helper function if you don't already have one
fn create_test_metadata() -> TokenMetadata {
    TokenMetadata {
        title: "Test Token".to_string(),
        description: Some("Test Description".to_string()),
        media: None,
        media_hash: None,
        copies: Some(1000),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        extra: None,
    }
}
