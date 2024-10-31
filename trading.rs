// actions/trading.rs

use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SwapResult {
    pub tokens_out: Balance,
    pub price_impact: f64,
    pub fee_amount: Balance,
}

#[near_bindgen]
impl TokenBlocks {
    #[payable]
    pub fn swap_native_for_tokens(
        &mut self,
        token_id: TokenId,
        min_tokens_out: U128
    ) -> SwapResult {
        let native_in = env::attached_deposit();
        let buyer = env::predecessor_account_id();
        
        assert!(native_in > 0, "Must attach native tokens");
        
        let mut pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        // Calculate swap details
        let fee_amount = native_in * pool.fee_rate / 10_000; // e.g., 0.3% fee
        let native_in_after_fee = native_in - fee_amount;
        
        // Calculate tokens out using constant product formula
        let tokens_out = pool.calculate_tokens_out(native_in_after_fee);
        assert!(
            tokens_out >= min_tokens_out.0,
            "Slippage tolerance exceeded"
        );
        
        // Calculate price impact
        let price_impact = pool.calculate_price_impact(native_in_after_fee, true);
        
        // Update pool reserves
        pool.native_reserve += native_in_after_fee;
        pool.token_reserve -= tokens_out;
        pool.total_fees += fee_amount;
        pool.update_volume(native_in);
        
        // Update pool state
        self.pools.insert(&token_id, &pool);
        
        // Transfer tokens to buyer
        let mut token = self.tokens.get(&token_id)
            .expect("Token not found");
        token.circulating_supply += tokens_out;
        self.tokens.insert(&token_id, &token);
        
        SwapResult {
            tokens_out,
            price_impact,
            fee_amount,
        }
    }
    
    #[payable]
    pub fn swap_tokens_for_native(
        &mut self,
        token_id: TokenId,
        token_amount: U128,
        min_native_out: U128
    ) -> SwapResult {
        let tokens_in = token_amount.0;
        let seller = env::predecessor_account_id();
        
        let mut pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        // Calculate swap
        let fee_amount = tokens_in * pool.fee_rate / 10_000;
        let tokens_in_after_fee = tokens_in - fee_amount;
        
        // Calculate native out using constant product formula
        let native_out = pool.calculate_native_out(tokens_in_after_fee);
        assert!(
            native_out >= min_native_out.0,
            "Slippage tolerance exceeded"
        );
        
        // Calculate price impact
        let price_impact = pool.calculate_price_impact(tokens_in_after_fee, false);
        
        // Update pool reserves
        pool.token_reserve += tokens_in_after_fee;
        pool.native_reserve -= native_out;
        pool.total_fees += fee_amount;
        pool.update_volume(tokens_in);
        
        // Update pool state
        self.pools.insert(&token_id, &pool);
        
        // Transfer native tokens to seller
        Promise::new(seller).transfer(native_out);
        
        SwapResult {
            tokens_out: native_out,
            price_impact,
            fee_amount,
        }
    }
    
    pub fn add_liquidity(
        &mut self,
        token_id: TokenId,
        token_amount: U128
    ) -> Balance {
        let native_deposit = env::attached_deposit();
        let provider = env::predecessor_account_id();
        
        let mut pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        // If first liquidity provision, accept any ratio
        if pool.native_reserve == 0 {
            return pool.initialize_liquidity(token_amount.0, native_deposit);
        }
        
        // Calculate optimal amounts
        let optimal_native = pool.calculate_optimal_native(token_amount.0);
        assert!(
            native_deposit >= optimal_native,
            "Insufficient native tokens"
        );
        
        // Add liquidity
        let lp_tokens = pool.add_liquidity(token_amount.0, optimal_native);
        
        // Refund excess native tokens
        if native_deposit > optimal_native {
            Promise::new(provider).transfer(native_deposit - optimal_native);
        }
        
        // Update pool
        self.pools.insert(&token_id, &pool);
        
        lp_tokens
    }
    
    pub fn remove_liquidity(
        &mut self,
        token_id: TokenId,
        lp_tokens: U128,
        min_native: U128,
        min_tokens: U128
    ) -> (Balance, Balance) {
        let provider = env::predecessor_account_id();
        
        let mut pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        // Calculate amounts to return
        let (native_amount, token_amount) = pool.remove_liquidity(
            lp_tokens.0,
            min_native.0,
            min_tokens.0
        );
        
        // Update pool state
        self.pools.insert(&token_id, &pool);
        
        // Transfer assets to provider
        Promise::new(provider).transfer(native_amount);
        
        (native_amount, token_amount)
    }
    
    // View methods
    pub fn get_pool_info(&self, token_id: TokenId) -> PoolInfo {
        let pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        PoolInfo {
            token_reserve: pool.token_reserve.into(),
            native_reserve: pool.native_reserve.into(),
            total_volume: pool.total_volume.into(),
            total_fees: pool.total_fees.into(),
            fee_rate: pool.fee_rate,
            price: pool.get_current_price(),
        }
    }
    
    pub fn get_swap_estimate(
        &self,
        token_id: TokenId,
        amount_in: U128,
        is_native: bool
    ) -> SwapEstimate {
        let pool = self.pools.get(&token_id)
            .expect("Pool not found");
            
        let amount_in = amount_in.0;
        let fee_amount = amount_in * pool.fee_rate / 10_000;
        let amount_in_after_fee = amount_in - fee_amount;
        
        let amount_out = if is_native {
            pool.calculate_tokens_out(amount_in_after_fee)
        } else {
            pool.calculate_native_out(amount_in_after_fee)
        };
        
        let price_impact = pool.calculate_price_impact(amount_in_after_fee, is_native);
        
        SwapEstimate {
            amount_out: amount_out.into(),
            fee_amount: fee_amount.into(),
            price_impact,
        }
    }
}

// models/pool.rs
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, Timestamp};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub token_id: TokenId,
    pub token_reserve: Balance,
    pub native_reserve: Balance,
    pub total_volume: Balance,
    pub total_fees: Balance,
    pub fee_rate: u32,          // basis points (e.g., 30 = 0.3%)
    pub last_updated: Timestamp,
    pub volume_24h: Balance,
    pub last_volume_update: Timestamp,
}

impl Pool {
    pub fn new(token_id: TokenId, initial_token_reserve: Balance) -> Self {
        Self {
            token_id,
            token_reserve: initial_token_reserve,
            native_reserve: 0,
            total_volume: 0,
            total_fees: 0,
            fee_rate: 30, // 0.3%
            last_updated: env::block_timestamp(),
            volume_24h: 0,
            last_volume_update: env::block_timestamp(),
        }
    }

    pub fn initialize_liquidity(
        &mut self,
        token_amount: Balance,
        native_amount: Balance
    ) -> Balance {
        assert!(self.native_reserve == 0, "Pool already initialized");
        assert!(token_amount > 0 && native_amount > 0, "Zero amounts");
        
        self.token_reserve = token_amount;
        self.native_reserve = native_amount;
        self.last_updated = env::block_timestamp();
        
        // Initial LP tokens are sqrt(x * y)
        (token_amount as f64 * native_amount as f64).sqrt() as Balance
    }
    
    pub fn calculate_tokens_out(&self, native_in: Balance) -> Balance {
        // x * y = k formula
        // (x + Δx)(y - Δy) = xy
        // Solving for Δy (tokens_out)
        let k = self.token_reserve as u128 * self.native_reserve as u128;
        let new_native_reserve = self.native_reserve as u128 + native_in as u128;
        let new_token_reserve = k / new_native_reserve;
        self.token_reserve as u128 - new_token_reserve
    }
    
    pub fn calculate_native_out(&self, tokens_in: Balance) -> Balance {
        let k = self.token_reserve as u128 * self.native_reserve as u128;
        let new_token_reserve = self.token_reserve as u128 + tokens_in as u128;
        let new_native_reserve = k / new_token_reserve;
        self.native_reserve as u128 - new_native_reserve
    }
    
    pub fn calculate_price_impact(&self, amount_in: Balance, is_native: bool) -> f64 {
        let (reserve_in, reserve_out) = if is_native {
            (self.native_reserve, self.token_reserve)
        } else {
            (self.token_reserve, self.native_reserve)
        };
        
        let amount_out = if is_native {
            self.calculate_tokens_out(amount_in)
        } else {
            self.calculate_native_out(amount_in)
        };
        
        let initial_price = reserve_out as f64 / reserve_in as f64;
        let final_price = (reserve_out - amount_out) as f64 / 
                         (reserve_in + amount_in) as f64;
        
        ((final_price - initial_price) / initial_price * 100.0).abs()
    }
    
    pub fn update_volume(&mut self, amount: Balance) {
        self.total_volume += amount;
        
        let current_time = env::block_timestamp();
        let time_passed = current_time - self.last_volume_update;
        
        // Reset 24h volume if more than 24h passed
        if time_passed >= 24 * 60 * 60 * 1_000_000_000 {
            self.volume_24h = amount;
        } else {
            self.volume_24h += amount;
        }
        
        self.last_volume_update = current_time;
    }
    
    pub fn get_current_price(&self) -> f64 {
        self.native_reserve as f64 / self.token_reserve as f64
    }
}

// View structs for frontend
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    pub token_reserve: U128,
    pub native_reserve: U128,
    pub total_volume: U128,
    pub total_fees: U128,
    pub fee_rate: u32,
    pub price: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapEstimate {
    pub amount_out: U128,
    pub fee_amount: U128,
    pub price_impact: f64,
}
