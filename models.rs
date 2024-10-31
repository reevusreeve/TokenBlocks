use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, Balance};

pub type TokenId = u64;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum BlockPhase {
    AcceptingTokens,
    Voting,
    Public,
    Completed,
}

// Add other necessary struct definitions for Token, TokenMetadata, Block, etc. 