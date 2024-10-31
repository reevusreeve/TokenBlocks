use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use crate::*;

#[near_bindgen]
impl TokenBlocks {
    #[payable]
    pub fn create_token(
        &mut self,
        content_hash: String,
        metadata: TokenMetadata,
    ) -> TokenId {
        // Ensure sufficient payment for platform fee
        let deposit = env::attached_deposit();
        assert!(
            deposit >= self.platform_fee,
            "Insufficient deposit for token creation"
        );

        // Basic validation
        assert!(!content_hash.is_empty(), "Content hash cannot be empty");
        assert!(!metadata.title.is_empty(), "Token must have a title");

        // Create new token
        let token_id = self.token_counter;
        let token = Token::new(
            token_id,
            env::predecessor_account_id(),
            content_hash,
            metadata,
        );

        // Store token and update queue
        self.tokens.insert(&token_id, &token);
        self.token_queue.push(token_id);
        self.token_counter += 1;

        // Refund excess deposit
        if deposit > self.platform_fee {
            Promise::new(env::predecessor_account_id()).transfer(deposit - self.platform_fee);
        }

        token_id
    }

    // Internal method to process queued tokens into next block
    pub(crate) fn process_token_queue(&mut self) -> Vec<TokenId> {
        let current_time = env::block_timestamp();
        let mut processed_tokens = Vec::new();

        // Take tokens from queue and update their status
        while let Some(token_id) = self.token_queue.pop() {
            if let Some(mut token) = self.tokens.get(&token_id) {
                token.status = TokenStatus::InVoting;
                self.tokens.insert(&token_id, &token);
                processed_tokens.push(token_id);
            }
        }

        processed_tokens
    }

    // Admin function to update platform fee
    pub fn update_platform_fee(&mut self, new_fee: U128) {
        self.assert_owner();
        self.platform_fee = new_fee.0;
    }

    // View methods
    pub fn get_token(&self, token_id: TokenId) -> Option<TokenView> {
        self.tokens.get(&token_id).map(|token| (&token).into())
    }

    pub fn get_tokens_by_creator(&self, creator: AccountId) -> Vec<TokenView> {
        self.tokens
            .iter()
            .filter(|(_, token)| token.creator == creator)
            .map(|(_, token)| (&token).into())
            .collect()
    }

    // Helper methods
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only contract owner can call this method"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    fn test_create_token() {
        let context = get_context(AccountId::new_unchecked("creator.near".to_string()));
        testing_env!(context.build());

        let mut contract = TokenBlocks::new(
            AccountId::new_unchecked("owner.near".to_string()),
            None,
            None,
            Some(U128(1000)),
        );

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

        let token_id = contract.create_token(
            "ipfs://test".to_string(),
            metadata,
        );

        assert_eq!(token_id, 0);
        assert_eq!(contract.token_queue.len(), 1);
    }
}
