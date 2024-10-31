#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    fn test_validation() {
        // Test metadata validation
        let valid_metadata = TokenMetadata {
            title: "Test Token".to_string(),
            description: Some("Description".to_string()),
            media: Some("ipfs://hash".to_string()),
            media_hash: None,
            copies: Some(1000),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            extra: None,
        };
        assert!(Validation::assert_valid_metadata(&valid_metadata));

        // Test stake validation
        assert!(Validation::assert_valid_stake(
            100,  // stake amount
            10,   // min stake
            1000  // balance
        ));
    }

    #[test]
    fn test_math_calculations() {
        // Test share calculation
        let share = Math::calculate_share(100, 1000, 1000);
        assert_eq!(share, 100);

        // Test price impact
        let impact = Math::calculate_price_impact(100, 1000, 1000);
        assert!(impact > 0.0 && impact < 10.0);

        // Test liquidity share
        let lp_tokens = Math::calculate_liquidity_share(
            100,  // amount a
            100,  // amount b
            1000, // reserve a
            1000, // reserve b
            10000 // total supply
        );
        assert_eq!(lp_tokens, 1000);
    }

    #[test]
    fn test_time_functions() {
        let current_time = 1000000;
        let start_time = current_time - 1000;
        let end_time = current_time + 1000;

        assert!(Time::is_within_range(current_time, start_time, end_time));
        assert!(Time::assert_valid_time_range(start_time, end_time));

        let block_end = Time::get_block_end_time(current_time);
        assert_eq!(block_end, current_time + 300_000_000_000);
    }

    #[test]
    fn test_storage_calculations() {
        let test_data = "test data".to_string();
        let storage_usage = Storage::get_storage_usage(&test_data);
        assert!(storage_usage > 0);

        // Test storage coverage
        Storage::assert_storage_covered(
            storage_usage,
            storage_usage * Storage::STORAGE_PRICE_PER_BYTE
        );
    }
}
