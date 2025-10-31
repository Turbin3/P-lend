use pinocchio::pubkey::Pubkey;

use crate::helper::{account_init::StateDefinition, utils::DataLen};


pub struct LendingMarketState {
    pub version: u64,
    pub lending_market_owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub risk_council: Pubkey,
    pub emergency_mode: bool,
}

impl StateDefinition for LendingMarketState {
    const LEN: usize = 100;
    const SEED: &'static str = "lending_market";
}

impl LendingMarketState {
    pub fn new(lending_market_owner: Pubkey, quote_currency: [u8; 32], risk_council: Pubkey) -> Self {
        Self {
            version: 0,
            lending_market_owner,
            quote_currency,
            risk_council,
            emergency_mode: false,
        }
    }
}