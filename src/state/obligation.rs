use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;

use crate::StateDefinition;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct ObligationState {
    pub lending_market: Pubkey,
    pub owner: Pubkey,
    pub deposits: [ObligationCollateral; 8],
    pub borrows: [ObligationLiquidity; 5],
    pub health_factor: u64,
    pub liquidation_threshold: u64,
    pub last_update_slot: u64,
    pub _padding: u64,
}

impl StateDefinition for ObligationState {
    const LEN: usize = core::mem::size_of::<Self>();
    const SEED: &'static str = "obligation";
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObligationCollateral {
    pub deposit_reserve: Pubkey,
    pub market_value_sf: u128,
    pub deposited_amount: u64,
    pub _padding: u64,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObligationLiquidity {
    pub borrow_reserve: Pubkey,
    pub borrowed_amount: u128,
    pub market_value: u128,
}
