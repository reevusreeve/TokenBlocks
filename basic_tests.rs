use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;
use token_blocks::*;
use token_blocks::TokenMetadata;
use near_sdk::json_types::ValidAccountId;

fn get_context() -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .predecessor_account_id(ValidAccountId::try_from("user.near").unwrap())
        .current_account_id(ValidAccountId::try_from("contract.near").unwrap());
    builder
}

#[test]
fn test_token_creation() {
    let context = get_context();
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
    assert_eq!(stored_token.metadata.copies, Some(1000));
}