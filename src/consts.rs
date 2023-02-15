use std::str::FromStr;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref NEAR_SOCIAL_ACCOUNT_ID: std::collections::HashMap<&'static str, near_primitives::types::AccountId> =
        std::collections::HashMap::from([
            (
                "mainnet",
                near_primitives::types::AccountId::from_str("social.near").unwrap()
            ),
            (
                "testnet",
                near_primitives::types::AccountId::from_str("v1.social08.testnet").unwrap()
            ),
        ]);
}
