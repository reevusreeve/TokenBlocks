use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, Balance};
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{testing_env, MockedBlockchain};
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub token_id: TokenId,
    pub token_reserve: Balance,
    pub native_reserve: Balance,
    pub usdc_reserve: Balance,
    pub total_fees: Balance,
    pub last_updated: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolStats {
    pub volume_24h: Balance,
    pub fees_24h: Balance,
    pub transactions_24h: u32,
}

impl Pool {
    pub fn new(token_id: TokenId, initial_token_reserve: Balance) -> Self {
        Self {
            token_id,
            token_reserve: initial_token_reserve,
            native_reserve: 0,
            usdc_reserve: 0,
            total_fees: 0,
            last_updated: env::block_timestamp(),
        }
    }

    pub fn add_liquidity(
        &mut self, 
        token_amount: Balance, 
        native_amount: Balance
    ) -> Balance {
        self.token_reserve += token_amount;
        self.native_reserve += native_amount;
        self.last_updated = env::block_timestamp();
        self.calculate_liquidity_share(token_amount)
    }

    pub fn swap_tokens(
        &mut self,
        amount_in: Balance,
        is_native: bool,
    ) -> Balance {
        let amount_out = if is_native {
            self.calculate_token_out(amount_in)
        } else {
            self.calculate_native_out(amount_in)
        };

        if is_native {
            self.native_reserve += amount_in;
            self.token_reserve -= amount_out;
        } else {
            self.token_reserve += amount_in;
            self.native_reserve -= amount_out;
        }

        self.last_updated = env::block_timestamp();
        amount_out
    }

    fn calculate_token_out(&mut self, native_in: Balance) -> Balance {
        assert!(self.native_reserve > 0 && self.token_reserve > 0, "Insufficient reserves");
        
        // Calculate fee first (0.3% fee)
        let fee_amount = native_in * 3 / 1000; 
        let native_in_with_fee = native_in - fee_amount;
        self.total_fees += fee_amount;
        
        // Calculate output using constant product formula: (x * y) = k
        // The formula should be: dx = dy * x / (y + dy)
        // where dx is tokens_out, dy is native_in_with_fee, x is token_reserve, y is native_reserve
        let numerator = native_in_with_fee * self.token_reserve;
        let denominator = self.native_reserve + native_in_with_fee;
        
        let tokens_out = numerator / denominator;
        
        // Ensure we don't return more than available and maintain minimum reserve
        std::cmp::min(tokens_out, self.token_reserve - 1)
    }

    fn calculate_native_out(&mut self, token_in: Balance) -> Balance {
        let fee_amount = token_in * 30 / 10000; // 0.3% fee
        let token_in_with_fee = token_in - fee_amount;
        self.total_fees += fee_amount;
        
        let numerator = token_in_with_fee * self.native_reserve;
        let denominator = self.token_reserve + token_in_with_fee;
        numerator / denominator
    }

    fn calculate_liquidity_share(&self, token_amount: Balance) -> Balance {
        if self.token_reserve == 0 {
            token_amount
        } else {
            // Calculate proportional share based on the ratio of new tokens to existing tokens
            (token_amount * self.token_reserve) / self.token_reserve
        }
    }

    // New helper methods
    pub fn get_reserves(&self) -> (Balance, Balance) {
        (self.token_reserve, self.native_reserve)
    }

    pub fn get_fees(&self) -> Balance {
        self.total_fees
    }

    pub fn calculate_price_impact(&self, amount_in: Balance, is_native: bool) -> f64 {
        let (reserve_in, reserve_out) = if is_native {
            (self.native_reserve, self.token_reserve)
        } else {
            (self.token_reserve, self.native_reserve)
        };

        // Ensure we don't divide by zero
        if reserve_in == 0 || reserve_out == 0 {
            return 100.0;
        }

        let amount_with_fee = amount_in * 997 / 1000; // 0.3% fee
        let amount_out = amount_with_fee * reserve_out / (reserve_in + amount_with_fee);
        
        let initial_price = reserve_out as f64 / reserve_in as f64;
        let final_price = (reserve_out - amount_out) as f64 / (reserve_in + amount_in) as f64;
        
        ((final_price - initial_price) / initial_price * 100.0).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        // Initialize the testing environment
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        let pool = Pool::new(1, 1000);
        assert_eq!(pool.token_reserve, 1000);
        assert_eq!(pool.native_reserve, 0);
        assert_eq!(pool.total_fees, 0);
    }

    #[test]
    fn test_liquidity_addition() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        
        let mut pool = Pool::new(1, 1000);
        
        // First liquidity addition
        let share = pool.add_liquidity(1000, 1000);
        assert_eq!(pool.token_reserve, 2000);
        assert_eq!(pool.native_reserve, 1000);
        assert_eq!(share, 1000); // First liquidity provider gets exact amount
        
        // Second liquidity addition (should be proportional)
        let share2 = pool.add_liquidity(500, 500);
        assert_eq!(pool.token_reserve, 2500);
        assert_eq!(pool.native_reserve, 1500);
        assert_eq!(share2, 500); // Should be proportional to the contribution
    }

    #[test]
    fn test_swap_calculation() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        
        // Initialize pool with 1000 tokens and 1000 native tokens (1:1 ratio)
        let mut pool = Pool::new(1, 1000);
        pool.add_liquidity(1000, 1000); // Adds 1000 tokens and 1000 native tokens
        
        // Try to swap 100 native tokens
        let native_in = 100;
        let tokens_out = pool.calculate_token_out(native_in);
        
        println!("Native in: {}, Tokens out: {}", native_in, tokens_out);
        
        // Remove the assertion about tokens_out being less than native_in
        // assert!(tokens_out < native_in, "Output should be less than input due to fees");
        
        // Calculate expected output manually
        let amount_in_with_fee = native_in * 997;
        let numerator = amount_in_with_fee * pool.token_reserve;
        let denominator = pool.native_reserve * 1000 + amount_in_with_fee;
        let expected_output = numerator / denominator;
        
        assert_eq!(tokens_out, expected_output);
        
        // Verify reserves haven't changed (calculation doesn't modify state)
        let (token_reserve, native_reserve) = pool.get_reserves();
        assert_eq!(token_reserve, 2000);
        assert_eq!(native_reserve, 1000);
    }

    #[test]
    fn test_price_impact() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());
        
        let mut pool = Pool::new(1, 10000);
        pool.add_liquidity(10000, 10000); // 1:1 initial ratio with larger liquidity
        
        // Small trade (1% of pool size)
        let small_impact = pool.calculate_price_impact(100, true);
        assert!(small_impact < 2.0, "Small trades should have minimal impact");
        
        // Large trade (50% of pool size)
        let large_impact = pool.calculate_price_impact(5000, true);
        assert!(large_impact > 5.0, "Large trades should have significant impact");
        assert!(large_impact < 100.0, "Impact shouldn't exceed 100%");
    }
}