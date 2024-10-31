use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;
use token_blocks::*;
use token_blocks::{TokenMetadata, ACCEPTING_TOKENS_DURATION, VOTING_DURATION, BLOCK_DURATION};
use near_sdk::json_types::ValidAccountId;

fn setup_test_context() -> VMContextBuilder {
    let mut context = VMContextBuilder::new();
    context
        .predecessor_account_id(ValidAccountId::try_from("user.near").unwrap())
        .current_account_id(ValidAccountId::try_from("contract.near").unwrap())
        .block_timestamp(0)
        .attached_deposit(0);
    context
}

#[test]
fn test_block_lifecycle() {
    let mut context = setup_test_context();
    testing_env!(context.build());

    let mut contract = TokenBlocks::new("owner.near".to_string());

    // Create test token
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

    let token_id = contract.create_token(metadata);
    assert!(contract.get_queued_tokens().contains(&token_id));

    // Start block
    contract.start_block();
    let block = contract.get_current_block().unwrap();
    assert_eq!(block.tokens.len(), 1);
    assert_eq!(block.phase, "AcceptingTokens");

    // Move to Voting phase
    context.block_timestamp(ACCEPTING_TOKENS_DURATION + 1);
    testing_env!(context.build());

    contract.update_block_phase();
    let block = contract.get_current_block().unwrap();
    assert_eq!(block.phase, "Voting");

    // Move to Public phase
    context.block_timestamp(ACCEPTING_TOKENS_DURATION + VOTING_DURATION + 1);
    testing_env!(context.build());

    contract.update_block_phase();
    let block = contract.get_current_block().unwrap();
    assert_eq!(block.phase, "Public");

    // Move to Completed phase
    context.block_timestamp(BLOCK_DURATION + 1);
    testing_env!(context.build());

    contract.update_block_phase();

    // At this point, current_block should be None
    assert!(
        contract.get_current_block().is_none(),
        "Current block should be None after completion"
    );
}

#[test]
fn test_token_creation() {
    let context = setup_test_context();
    testing_env!(context.build());

    let mut contract = TokenBlocks::new("owner.near".to_string());

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

    let token_id = contract.create_token(metadata);
    let stored_token = contract.get_token(token_id).unwrap();
    assert_eq!(stored_token.metadata.title, "Test Token");
}