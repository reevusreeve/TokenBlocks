// actions/purchase.rs

use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use crate::*;

#[near_bindgen]
impl TokenBlocks {
    #[payable]
    pub fn purchase_with_native(
        &mut self,
        token_id: TokenId,
        amount: U128
    ) -> Balance {
        let payment = env::attached_deposit();
        let buyer = env::predecessor_account_id();
        
        self.process_purchase(token_id, amount.0, buyer, Some(payment), None)
    }

    #[payable]
    pub fn purchase_with_usdc(
        &mut self,
        token_id: TokenId,
        amount: U128,
        usdc_amount: U128
    ) -> Balance {
        let buyer = env::predecessor_account_id();
        
        // USDC transfer would be handled via ft_transfer_call
        self.process_purchase(token_id, amount.0, buyer, None, Some(usdc_amount.0))
    }

    fn process_purchase(
        &mut self,
        token_id: TokenId,
        amount: Balance,
        buyer: AccountId,
        native_payment: Option<Balance>,
        usdc_payment: Option<Balance>,
    ) -> Balance {
        // Validate purchase phase
        self.assert_valid_purchase_phase(buyer.clone());

        // Get and validate token
        let mut token = self.tokens.get(&token_id)
            .expect("Token not found");
        assert_eq!(token.status, TokenStatus::Winner, "Token not available for purchase");

        // Check available amount
        let available = token.available_for_purchase();
        assert!(amount <= available, "Insufficient tokens available");

        // Process payment and calculate tokens
        let tokens_to_buyer = if let Some(native_payment) = native_payment {
            self.process_native_payment(token_id, amount, native_payment)
        } else if let Some(usdc_payment) = usdc_payment {
            self.process_usdc_payment(token_id, amount, usdc_payment)
        } else {
            env::panic_str("Invalid payment method");
        };

        // Update token circulating supply
        token.circulating_supply += tokens_to_buyer;
        self.tokens.insert(&token_id, &token);

        // Update pool if necessary
        self.update_pool(token_id, tokens_to_buyer, native_payment, usdc_payment);

        tokens_to_buyer
    }

    fn process_native_payment(
        &mut self,
        token_id: TokenId,
        amount: Balance,
        payment: Balance
    ) -> Balance {
        // Calculate price using pool ratio
        let pool = self.pools.get(&token_id)
            .expect("Pool not found");
        let required_payment = pool.calculate_native_required(amount);
        assert!(payment >= required_payment, "Insufficient payment");

        // Return excess payment
        if payment > required_payment {
            Promise::new(env::predecessor_account_id())
                .transfer(payment - required_payment);
        }

        amount
    }

    fn process_usdc_payment(
        &mut self,
        token_id: TokenId,
        amount: Balance,
        usdc_amount: Balance
    ) -> Balance {
        // Similar to native payment but with USDC
        // Would need to handle USDC price calculations
        amount
    }

    fn update_pool(
        &mut self,
        token_id: TokenId,
        amount: Balance,
        native_payment: Option<Balance>,
        usdc_payment: Option<Balance>
    ) {
        let mut pool = self.pools.get(&token_id)
            .expect("Pool not found");

        // Calculate and add 5% to pool
        let pool_contribution = amount * 5 / 100;
        
        if let Some(native_payment) = native_payment {
            pool.add_liquidity(pool_contribution, native_payment * 5 / 100);
        } else if let Some(usdc_payment) = usdc_payment {
            pool.add_usdc_liquidity(pool_contribution, usdc_payment * 5 / 100);
        }

        self.pools.insert(&token_id, &pool);
    }

    fn assert_valid_purchase_phase(&self, buyer: AccountId) {
        let block = self.current_block.as_ref()
            .expect("No active block");
        
        let current_time = env::block_timestamp();
        let is_priority = block.is_priority_phase(current_time);
        
        if is_priority {
            // Check if buyer is a voter during priority phase
            assert!(
                self.is_voter(&buyer),
                "Only voters can purchase during priority phase"
            );
        } else {
            assert!(
                current_time < block.end_time + 300_000_000_000,
                "Purchase phase ended"
            );
        }
    }

    fn is_voter(&self, account_id: &AccountId) -> bool {
        self.stakes.get(account_id)
            .map(|stake_info| !stake_info.stakes.is_empty())
            .unwrap_or(false)
    }
}
