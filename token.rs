use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance};
use near_sdk::json_types::U128;
use crate::models::TokenId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenStatus {
    Queued,
    InVoting,
    Winner,
    Lost,
    Trading,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub id: TokenId,
    pub creator: AccountId,
    pub content_hash: String,      // IPFS/Arweave hash
    pub created_at: u64,
    pub total_supply: Balance,
    pub circulating_supply: Balance,
    pub pool_reserve: Balance,     // 20% of total supply
    pub status: TokenStatus,
    pub metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: String,
    pub description: Option<String>,
    pub media: Option<String>,     // IPFS/Arweave hash for media
    pub media_hash: Option<String>,
    pub copies: Option<u64>,       // Number of tokens to create if won
    pub issued_at: Option<u64>,    // When token was created
    pub expires_at: Option<u64>,   // Optional expiration
    pub starts_at: Option<u64>,    // Optional start time
    pub extra: Option<String>,     // Optional extra metadata
}


impl Token {
    pub fn new(
        id: TokenId,
        creator: AccountId,
        content_hash: String,
        metadata: TokenMetadata,
    ) -> Self {
        Self {
            id,
            creator,
            content_hash,
            created_at: env::block_timestamp(),
            total_supply: 0,        // Set when token wins
            circulating_supply: 0,
            pool_reserve: 0,        // 20% of total when created
            status: TokenStatus::Queued,
            metadata,
        }
    }

    pub fn initialize_supply(&mut self, total_supply: Balance) {
        assert_eq!(self.total_supply, 0, "Supply already initialized");
        self.total_supply = total_supply;
        self.pool_reserve = total_supply / 5;  // 20% reserve
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TokenStatus::InVoting | TokenStatus::Winner)
    }

    pub fn available_for_purchase(&self) -> Balance {
        self.total_supply - self.circulating_supply - self.pool_reserve
    }
}

// View structure for frontend
#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenView {
    pub id: TokenId,
    pub creator: AccountId,
    pub content_hash: String,
    pub created_at: u64,
    pub total_supply: U128,
    pub circulating_supply: U128,
    pub pool_reserve: U128,
    pub status: TokenStatus,
    pub metadata: TokenMetadata,
}

impl From<&Token> for TokenView {
    fn from(token: &Token) -> Self {
        Self {
            id: token.id,
            creator: token.creator.clone(),
            content_hash: token.content_hash.clone(),
            created_at: token.created_at,
            total_supply: U128::from(token.total_supply),
            circulating_supply: U128::from(token.circulating_supply),
            pool_reserve: U128::from(token.pool_reserve),
            status: token.status.clone(),
            metadata: token.metadata.clone(),
        }
    }
}
