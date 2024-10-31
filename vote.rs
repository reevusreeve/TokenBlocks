// actions/vote.rs

use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use crate::*;

#[near_bindgen]
impl TokenBlocks {
    #[payable]
    pub fn vote(&mut self, token_id: TokenId) -> bool {
        let stake_amount = env::attached_deposit();
        let voter = env::predecessor_account_id();

        // Validate voting conditions
        self.assert_active_voting_phase();
        assert!(stake_amount >= self.min_stake, "Stake too low");

        // Get token and validate
        let token = self.tokens.get(&token_id)
            .expect("Token not found");
        assert_eq!(token.status, TokenStatus::InVoting, "Token not in voting phase");

        // Record vote
        let mut vote_info = self.votes.get(&token_id)
            .unwrap_or_else(|| VoteInfo::new());
        vote_info.add_vote(&voter, stake_amount);
        self.votes.insert(&token_id, &vote_info);

        // Record stake
        let mut stake_info = self.stakes.get(&voter)
            .unwrap_or_else(|| StakeInfo::new(voter.clone()));
        stake_info.add_stake(token_id, stake_amount);
        self.stakes.insert(&voter, &stake_info);

        // Update block stats
        if let Some(ref mut block) = self.current_block {
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
    
        // Sort tokens by the number of votes
        token_votes.sort_by(|a, b| b.1.cmp(&a.1));
        let winners: Vec<TokenId> = token_votes.iter()
            .take(MAX_WINNERS as usize)
            .map(|(id, _)| *id)
            .collect();
    
        // Process each token
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

    fn return_stakes(&mut self, token_id: TokenId) {
        if let Some(vote_info) = self.votes.get(&token_id) {
            for (voter, amount) in vote_info.voters.iter() {
                Promise::new(voter.clone())
                    .transfer(*amount);
            }
        }
    }

    // Helper methods
    fn assert_active_voting_phase(&self) {
        assert!(self.current_block.is_some(), "No active block");
        let block = self.current_block.as_ref().unwrap();
        assert!(
            block.is_voting_phase(env::block_timestamp()),
            "Not in voting phase"
        );
    }

    fn is_voting_phase_ended(&self) -> bool {
        if let Some(block) = &self.current_block {
            let voting_end_time = block.start_time + block.accepting_tokens_duration + block.voting_duration;
            env::block_timestamp() >= voting_end_time
        } else {
            false
        }
    }
}