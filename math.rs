// utils/math.rs

use near_sdk::Balance;
use uint::construct_uint;

// Define U256 for high precision calculations
construct_uint! {
    pub struct U256(4);
}

pub struct Math;

impl Math {
    // Constants for calculations
    pub const PRECISION: u128 = 1_000_000;  // 6 decimal places
    pub const FEE_DENOMINATOR: u128 = 10_000; // For basis points (100% = 10000)
    pub const PRICE_PRECISION: u128 = 1_000_000_000; // 9 decimal places for price
    
    /// Calculates share of total based on contribution
    /// Returns amount * total_supply / total_amount with proper rounding
    pub fn calculate_share(
        amount: Balance,
        total_amount: Balance,
        total_supply: Balance
    ) -> Balance {
        if total_amount == 0 || total_supply == 0 {
            amount
        } else {
            let temp = U256::from(amount) * U256::from(total_supply);
            (temp / U256::from(total_amount)).as_u128()
        }
    }

    /// Calculates optimal swap amount to maintain pool ratio
    pub fn calculate_optimal_swap(
        amount_a: Balance,
        reserve_a: Balance,
        reserve_b: Balance
    ) -> Balance {
        if reserve_a == 0 || reserve_b == 0 {
            return 0;
        }
        
        let amount_with_fee = amount_a * 997; // 0.3% fee
        let numerator = amount_with_fee * reserve_b;
        let denominator = reserve_a * 1000 + amount_with_fee;
        numerator / denominator
    }

    /// Calculates price impact as a percentage
    pub fn calculate_price_impact(
        amount_in: Balance,
        reserve_in: Balance,
        reserve_out: Balance
    ) -> f64 {
        if reserve_in == 0 || reserve_out == 0 {
            return 0.0;
        }

        let amount_with_fee = amount_in * 997 / 1000; // 0.3% fee
        let numerator = amount_with_fee * reserve_out;
        let denominator = reserve_in * 1000 + amount_with_fee * 997;
        let amount_out = numerator / denominator;
        
        let initial_price = reserve_out as f64 / reserve_in as f64;
        let final_price = (reserve_out - amount_out) as f64 / 
                         (reserve_in + amount_in) as f64;
        
        ((final_price - initial_price) / initial_price * 100.0).abs()
    }

    /// Calculates liquidity provider tokens for pool contribution
    pub fn calculate_liquidity_tokens(
        amount_a: Balance,
        amount_b: Balance,
        reserve_a: Balance,
        reserve_b: Balance,
        total_supply: Balance
    ) -> Balance {
        if total_supply == 0 {
            // Initial liquidity provision
            (amount_a as f64 * amount_b as f64).sqrt() as Balance
        } else {
            // Subsequent liquidity provision
            std::cmp::min(
                amount_a * total_supply / reserve_a,
                amount_b * total_supply / reserve_b
            )
        }
    }

    /// Calculates the proportion of tokens for removal
    pub fn calculate_remove_liquidity(
        lp_tokens: Balance,
        total_supply: Balance,
        reserve_a: Balance,
        reserve_b: Balance
    ) -> (Balance, Balance) {
        assert!(lp_tokens <= total_supply, "Insufficient LP tokens");
        
        let token_a_amount = lp_tokens * reserve_a / total_supply;
        let token_b_amount = lp_tokens * reserve_b / total_supply;
        
        (token_a_amount, token_b_amount)
    }

    /// Constant Product Formula (x * y = k)
    pub fn constant_product(
        x: Balance,
        y: Balance,
        dx: Balance,
        fee_numerator: u32,
        fee_denominator: u32
    ) -> Balance {
        let x_u256 = U256::from(x);
        let y_u256 = U256::from(y);
        let dx_u256 = U256::from(dx);
        let fee_num = U256::from(fee_numerator);
        let fee_den = U256::from(fee_denominator);

        let dx_with_fee = dx_u256 * fee_num / fee_den;
        let numerator = dx_with_fee * y_u256;
        let denominator = x_u256 + dx_with_fee;

        (numerator / denominator).as_u128()
    }

    /// Calculate square root using Newton's method
    pub fn sqrt(x: Balance) -> Balance {
        if x == 0 {
            return 0;
        }

        let mut z = x;
        let mut y = x / 2 + 1;
        while y < z {
            z = y;
            y = (x / y + y) / 2;
        }
        z
    }

    /// Calculates fee amount from total amount
    pub fn calculate_fee(
        amount: Balance,
        fee_basis_points: u32
    ) -> Balance {
        amount * fee_basis_points as u128 / Math::FEE_DENOMINATOR
    }

    /// Helper to calculate percentage
    pub fn calculate_percentage(
        amount: Balance,
        percentage: u32
    ) -> Balance {
        amount * percentage as u128 / 100
    }
    
    /// Slippage check
    pub fn check_slippage(
        expected: Balance,
        actual: Balance,
        slippage_bps: u32
    ) -> bool {
        let min_amount = expected * (Math::FEE_DENOMINATOR - slippage_bps as u128) 
            / Math::FEE_DENOMINATOR;
        actual >= min_amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_calculation() {
        // Test basic share calculation
        assert_eq!(
            Math::calculate_share(500, 1000, 1000),
            500,
            "Basic share calculation failed"
        );

        // Test with zero values
        assert_eq!(
            Math::calculate_share(100, 0, 1000),
            100,
            "Zero total amount handling failed"
        );

        // Test with large numbers
        let large_amount = 1_000_000_000_000_000;
        assert!(
            Math::calculate_share(large_amount, large_amount * 2, large_amount) > 0,
            "Large number handling failed"
        );
    }

    #[test]
    fn test_price_impact() {
        let impact = Math::calculate_price_impact(1000, 10000, 10000);
        assert!(impact > 0.0 && impact < 20.0, "Price impact calculation failed");

        // Test with small amounts
        let small_impact = Math::calculate_price_impact(100, 10000, 10000);
        assert!(small_impact < impact, "Small amount should have less impact");
    }

    #[test]
    fn test_liquidity_calculations() {
        // Test initial liquidity
        let initial_lp = Math::calculate_liquidity_tokens(1000, 1000, 0, 0, 0);
        assert!(initial_lp > 0, "Initial liquidity calculation failed");

        // Test subsequent liquidity
        let subsequent_lp = Math::calculate_liquidity_tokens(
            1000, 1000, 2000, 2000, 1000
        );
        assert_eq!(subsequent_lp, 500, "Subsequent liquidity calculation failed");
    }

    #[test]
    fn test_constant_product() {
        let dy = Math::constant_product(1000, 1000, 100, 997, 1000);
        assert!(dy > 0 && dy < 100, "Constant product calculation failed");
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(Math::sqrt(100), 10, "Square root calculation failed");
        assert_eq!(Math::sqrt(0), 0, "Square root of zero failed");
        assert_eq!(Math::sqrt(1), 1, "Square root of one failed");
    }

    #[test]
    fn test_fee_calculation() {
        // Test 0.3% fee
        assert_eq!(
            Math::calculate_fee(1000, 30),
            3,
            "Fee calculation failed"
        );

        // Test max fee (100%)
        assert_eq!(
            Math::calculate_fee(1000, 10000),
            1000,
            "Max fee calculation failed"
        );
    }

    #[test]
    fn test_slippage_check() {
        // Test 1% slippage
        assert!(
            Math::check_slippage(1000, 995, 100),
            "Valid slippage check failed"
        );

        // Test slippage exceeded
        assert!(
            !Math::check_slippage(1000, 900, 100),
            "Invalid slippage check failed"
        );
    }
}
