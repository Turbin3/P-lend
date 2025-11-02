use pinocchio::pubkey::Pubkey;

use crate::helper::{account_init::StateDefinition, utils::DataLen};
use bytemuck::{Pod,Zeroable};

#[repr(C,packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LendingMarketState {
    pub version: u64,
    pub lending_market_owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub risk_council: Pubkey,
    pub emergency_mode: u8,
}

impl StateDefinition for LendingMarketState {
    const LEN: usize = core::mem::size_of::<Self>();
    const SEED: &'static str = "lending_market";
}

impl DataLen for LendingMarketState {
    const LEN: usize = <Self as StateDefinition>::LEN;
}

impl LendingMarketState {
    pub fn new(
        lending_market_owner: Pubkey,
        quote_currency: [u8; 32],
        risk_council: Pubkey,
    ) -> Self {
        Self {
            version: 0,
            lending_market_owner,
            quote_currency,
            risk_council,
            emergency_mode: 0,
        }
    }
}
