use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};
use near_sdk::json_types::U128;

pub mod models;
pub use crate::models::{
    Token, TokenId, TokenMetadata, TokenStatus,
    Block, BlockPhase, BlockView, 
    VoteInfo, StakeInfo,
    TokenView,
};

pub const ACCEPTING_TOKENS_DURATION: u64 = 60_000_000_000; // 1 minute
pub const VOTING_DURATION: u64 = 120_000_000_000; // 2 minutes
pub const BLOCK_DURATION: u64 = 300_000_000_000; // 5 minutes in nanoseconds
const PUBLIC_DURATION: u64 = 120_000_000_000; // 2 minutes
const MIN_STAKE_AMOUNT: Balance = 1_000_000_000_000_000_000_000; // 1 NEAR
const MAX_WINNERS: u8 = 10;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct TokenBlocks {
    pub owner_id: AccountId,
    pub token_counter: TokenId,
    pub tokens: UnorderedMap<TokenId, Token>,
    pub current_block: Option<Block>,
    pub token_queue: Vec<TokenId>,
    pub votes: UnorderedMap<TokenId, VoteInfo>,
    pub stakes: UnorderedMap<AccountId, StakeInfo>,
    pub min_stake: Balance,
}

#[near_bindgen]
impl TokenBlocks {
    #[init]
    pub fn new(owner_id: String) -> Self {
        Self {
            owner_id: AccountId::try_from(owner_id).unwrap(),
            token_counter: 0,
            tokens: UnorderedMap::new(b"t"),
            current_block: None,
            token_queue: Vec::new(),
            votes: UnorderedMap::new(b"v"),
            stakes: UnorderedMap::new(b"s"),
            min_stake: MIN_STAKE_AMOUNT,
        }
    }

    pub fn create_token(&mut self, metadata: TokenMetadata) -> TokenId {
        let token_id = self.token_counter;
        let token = Token::new(
            token_id,
            env::predecessor_account_id(),
            "ipfs://".to_string(),
            metadata,
        );

        self.tokens.insert(&token_id, &token);
        self.token_counter += 1;
        
        if let Some(ref mut block) = self.current_block {
            if block.is_accepting_tokens(env::block_timestamp()) {
                block.add_token(token_id);
            } else {
                self.token_queue.push(token_id);
            }
        } else {
            self.token_queue.push(token_id);
        }
        
        token_id
    }

    pub fn start_block(&mut self) {
        assert!(self.current_block.is_none(), "Block already in progress");
        assert!(!self.token_queue.is_empty(), "No tokens in queue");
        
        let start_time = env::block_timestamp();
        let mut block = Block::new(
            start_time,
            ACCEPTING_TOKENS_DURATION,
            VOTING_DURATION,
            PUBLIC_DURATION,
            self.min_stake,
            MAX_WINNERS,
        );

        while let Some(token_id) = self.token_queue.pop() {
            block.add_token(token_id);
        }

        self.current_block = Some(block);
    }

    pub fn update_block_phase(&mut self) {
        if let Some(ref mut block) = self.current_block {
            let previous_phase = block.phase.clone();
            block.update_phase(env::block_timestamp());
    
            // Only update token statuses if the phase has changed
            if block.phase != previous_phase {
                self.update_tokens_status(&block.tokens, &block.phase);
            }
    
            if matches!(block.phase, BlockPhase::Completed) {
                self.current_block = None;
                if !self.token_queue.is_empty() {
                    self.start_block();
                }
            }
        }
    }

    #[payable]
    pub fn vote(&mut self, token_id: TokenId) -> bool {
        let stake_amount = env::attached_deposit();
        let voter = env::predecessor_account_id();

        self.assert_active_voting_phase();
        assert!(stake_amount >= MIN_STAKE_AMOUNT, "Stake too low");

        let token = self.tokens.get(&token_id)
            .expect("Token not found");
        assert_eq!(token.status, TokenStatus::InVoting, "Token not in voting phase");

        let mut vote_info = self.votes.get(&token_id)
            .unwrap_or_else(|| VoteInfo::new());
        vote_info.add_vote(&voter, stake_amount);
        self.votes.insert(&token_id, &vote_info);

        let mut stake_info = self.stakes.get(&voter)
            .unwrap_or_else(|| StakeInfo::new(voter.clone()));
        stake_info.add_stake(token_id, stake_amount);
        self.stakes.insert(&voter, &stake_info);

        if let Some(block) = &mut self.current_block {
            block.total_stakes += stake_amount;
        }

        true
    }

    pub fn process_voting_results(&mut self) {
        assert!(self.is_voting_phase_ended(), "Voting phase not ended");
        
        // Move the block out of `self.current_block` using `take()`
        let block = self.current_block.take()
            .expect("No active block");
    
        // Now, you can mutably borrow `self` without conflicts
        let mut token_votes: Vec<(TokenId, Balance)> = block.tokens.iter()
            .map(|&token_id| {
                let votes = self.votes.get(&token_id)
                    .map(|v| v.total_votes)
                    .unwrap_or(0);
                (token_id, votes)
            })
            .collect();
    
        token_votes.sort_by(|a, b| b.1.cmp(&a.1));
        let winners: Vec<TokenId> = token_votes.iter()
            .take(MAX_WINNERS as usize)
            .map(|(id, _)| *id)
            .collect();
    
        for &token_id in &block.tokens {
            let mut token = self.tokens.get(&token_id)
                .expect("Token not found");
    
            if winners.contains(&token_id) {
                token.status = TokenStatus::Winner;
                token.initialize_supply(1_000_000);
            } else {
                token.status = TokenStatus::Lost;
                self.return_stakes(token_id);
            }
    
            self.tokens.insert(&token_id, &token);
        }
    
        // Optionally, start a new block if there are tokens in the queue
        if !self.token_queue.is_empty() {
            self.start_block();
        } else {
            self.current_block = None;
        }
    }

    // View methods
    pub fn get_token(&self, token_id: TokenId) -> Option<TokenView> {
        self.tokens.get(&token_id).map(|token: Token| (&token).into())
    }
    
    pub fn get_tokens_by_creator(&self, creator: AccountId) -> Vec<TokenView> {
        self.tokens
            .iter()
            .filter(|(_, token)| token.creator == creator)
            .map(|(_, token): (TokenId, Token)| (&token).into())
            .collect()
    }

    pub fn get_current_block(&self) -> Option<BlockView> {
        self.current_block.as_ref().map(BlockView::from)
    }

    pub fn get_queued_tokens(&self) -> Vec<TokenId> {
        self.token_queue.clone()
    }

    pub fn get_block_info(&self) -> (u64, Balance, u8) {
        (BLOCK_DURATION, MIN_STAKE_AMOUNT, MAX_WINNERS)
    }

    pub fn get_votes(&self, token_id: TokenId) -> Option<U128> {
        self.votes.get(&token_id)
            .map(|v| U128(v.total_votes))
    }

    pub fn get_user_stakes(&self, account_id: AccountId) -> Option<U128> {
        self.stakes.get(&account_id)
            .map(|s| U128(s.total_staked))
    }

    // Helper methods
    fn return_stakes(&mut self, token_id: TokenId) {
        if let Some(vote_info) = self.votes.get(&token_id) {
            for (voter, amount) in vote_info.voters.iter() {
                Promise::new(voter).transfer(amount);
            }
        }
    }

    fn assert_active_voting_phase(&self) {
        assert!(self.current_block.is_some(), "No active block");
        let block = self.current_block.as_ref().unwrap();
        assert!(
            matches!(block.phase, BlockPhase::Voting),
            "Not in voting phase"
        );
    }

    fn is_voting_phase_ended(&self) -> bool {
        if let Some(block) = &self.current_block {
            env::block_timestamp() >= block.voting_end_time
        } else {
            false
        }
    }

    fn update_tokens_status(&mut self, token_ids: &[TokenId], phase: &BlockPhase) {
        for &token_id in token_ids {
            if let Some(mut token) = self.tokens.get(&token_id) {
                token.status = match phase {
                    BlockPhase::AcceptingTokens => TokenStatus::Pending,
                    BlockPhase::Voting => TokenStatus::InVoting,
                    BlockPhase::Public => TokenStatus::Public,
                    BlockPhase::Completed => token.status, // Keep existing status
                };
                self.tokens.insert(&token_id, &token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;
    use near_sdk::json_types::ValidAccountId;

    fn get_context() -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .predecessor_account_id(ValidAccountId::try_from("owner.near".to_string()).unwrap())
            .current_account_id(ValidAccountId::try_from("contract.near".to_string()).unwrap());
        builder
    }

    #[test]
    fn test_create_token() {
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

        let token_id = contract.create_token(metadata.clone());
        assert_eq!(token_id, 0);

        let token = contract.get_token(token_id).unwrap();
        assert_eq!(token.metadata.title, "Test Token");
    }

    #[test]
    fn test_block_lifecycle() {
        let mut context = get_context();
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
        assert!(contract.get_queued_tokens().contains(&token_id));
    
        contract.start_block();
        let block = contract.get_current_block().unwrap();
        assert_eq!(block.tokens.len(), 1);
    
        // Simulate passage of time to reach the Public phase
        let voting_end_time = ACCEPTING_TOKENS_DURATION + VOTING_DURATION; // 180_000_000_000 ns
        let _public_end_time_ = voting_end_time + PUBLIC_DURATION; // 300_000_000_000 ns
        let public_phase_time = voting_end_time + 1; // 180_000_000_001 ns
        context.block_timestamp(public_phase_time);
        testing_env!(context.build());
    
        contract.update_block_phase();
        let block = contract.get_current_block().unwrap();
    
        // Corrected assertion
        assert!(block.phase == "Public", "Block should be in Public phase");
    }

    #[test]
    fn test_voting() {
        let mut context = get_context();
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
        contract.start_block();

        // Move time forward past accepting tokens phase
        let voting_start_time = ACCEPTING_TOKENS_DURATION + 1;
        context.block_timestamp(voting_start_time);
        testing_env!(context.build());
        
        // Update the block phase
        contract.update_block_phase();
        
        // Update token status to InVoting
        if let Some(mut token) = contract.tokens.get(&token_id) {
            token.status = TokenStatus::InVoting;
            contract.tokens.insert(&token_id, &token);
        }

        context.attached_deposit(MIN_STAKE_AMOUNT);
        testing_env!(context.build());

        let vote_result = contract.vote(token_id);
        assert!(vote_result);

        let votes = contract.get_votes(token_id).unwrap();
        assert_eq!(votes.0, MIN_STAKE_AMOUNT);
    }
}