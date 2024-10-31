use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, Balance};
use near_sdk::collections::UnorderedMap;
use crate::TokenId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct VoteInfo {
    pub total_votes: Balance,
    pub voters: UnorderedMap<AccountId, Balance>,
}

impl VoteInfo {
    pub fn new() -> Self {
        Self {
            total_votes: 0,
            voters: UnorderedMap::new(b"v"),
        }
    }

    pub fn add_vote(&mut self, voter: &AccountId, amount: Balance) {
        let current = self.voters.get(voter).unwrap_or(0);
        self.voters.insert(voter, &(current + amount));
        self.total_votes += amount;
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakeInfo {
    pub account_id: AccountId,
    pub stakes: UnorderedMap<TokenId, Balance>,
    pub total_staked: Balance,
}

impl StakeInfo {
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            stakes: UnorderedMap::new(b"s"),
            total_staked: 0,
        }
    }

    pub fn add_stake(&mut self, token_id: TokenId, amount: Balance) {
        let current = self.stakes.get(&token_id).unwrap_or(0);
        self.stakes.insert(&token_id, &(current + amount));
        self.total_staked += amount;
    }
}