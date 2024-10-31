use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::U128;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum TokenStatus {
    Created,
    Pending,  // Add this variant
    InVoting,
    Public,
    Winner,
    Lost,
    Voting,
    Finished
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum BlockPhase {
    Voting,
    Finished,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Block {
    pub start_time: u64,
    pub accepting_tokens_duration: u64,
    pub voting_duration: u64,
    pub public_duration: u64,
    pub min_stake: Balance,
    pub max_winners: u8,
    pub tokens: Vec<TokenId>,
    pub total_stakes: Balance,
    pub phase: BlockPhase,
    pub voting_end_time: u64, // Added field
}

impl Block {
    pub fn new(
        start_time: u64,
        accepting_tokens_duration: u64,
        voting_duration: u64,
        public_duration: u64,
        min_stake: Balance,
        max_winners: u8,
    ) -> Self {
        let voting_end_time = start_time + accepting_tokens_duration + voting_duration;
        Self {
            start_time,
            accepting_tokens_duration,
            voting_duration,
            public_duration,
            min_stake,
            max_winners,
            tokens: Vec::new(),
            total_stakes: 0,
            phase: BlockPhase::AcceptingTokens,
            voting_end_time,
        }
    }

    pub fn add_token(&mut self, token_id: TokenId) {
        self.tokens.push(token_id);
    }

    pub fn update_phase(&mut self, current_time: u64) {
        let accepting_end = self.start_time + self.accepting_tokens_duration;
        let voting_end = accepting_end + self.voting_duration;
        let public_end = voting_end + self.public_duration;

        self.phase = if current_time < accepting_end {
            BlockPhase::AcceptingTokens
        } else if current_time < voting_end {
            BlockPhase::Voting
        } else if current_time < public_end {
            BlockPhase::Public
        } else {
            BlockPhase::Completed
        };
    }

    pub fn is_accepting_tokens(&self, current_time: u64) -> bool {
        matches!(self.phase, BlockPhase::AcceptingTokens)
            && current_time < self.start_time + self.accepting_tokens_duration
    }

    pub fn is_voting_phase(&self, current_time: u64) -> bool {
        matches!(self.phase, BlockPhase::Voting)
            && current_time >= self.start_time + self.accepting_tokens_duration
            && current_time < self.start_time + self.accepting_tokens_duration + self.voting_duration
    }

    pub fn is_public_phase(&self, current_time: u64) -> bool {
        matches!(self.phase, BlockPhase::Public)
            && current_time >= self.start_time + self.accepting_tokens_duration + self.voting_duration
            && current_time < self.start_time + self.accepting_tokens_duration + self.voting_duration + self.public_duration
    }
}

// Add BlockView
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BlockView {
    pub start_time: u64,
    pub accepting_tokens_duration: u64,
    pub voting_duration: u64,
    pub public_duration: u64,
    pub min_stake: U128,
    pub max_winners: u8,
    pub tokens: Vec<TokenId>,
    pub total_stakes: U128,
    pub phase: String,
}

impl From<&Block> for BlockView {
    fn from(block: &Block) -> Self {
        Self {
            start_time: block.start_time,
            accepting_tokens_duration: block.accepting_tokens_duration,
            voting_duration: block.voting_duration,
            public_duration: block.public_duration,
            min_stake: U128(block.min_stake),
            max_winners: block.max_winners,
            tokens: block.tokens.clone(),
            total_stakes: U128(block.total_stakes),
            phase: match block.phase {
                BlockPhase::AcceptingTokens => "AcceptingTokens".to_string(),
                BlockPhase::Voting => "Voting".to_string(),
                BlockPhase::Public => "Public".to_string(),
                BlockPhase::Completed => "Completed".to_string(),
                BlockPhase::Priority => "Priority".to_string(),
            },
        }
    }
}


