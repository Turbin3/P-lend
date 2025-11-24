use pinocchio::pubkey::Pubkey;

use crate::{
    helper::{account_init::StateDefinition, utils::DataLen},
    constants::RESERVE_SEED,
};
use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Pod, Zeroable, Debug, Clone, Copy, PartialEq)]
pub struct ReserveState {
    pub lending_market: Pubkey,
    pub mint: Pubkey,
    pub version: u64,
    pub available_liquidity: u64,
    pub total_supply: u64,
    pub total_borrows: u64,
    pub supply_index: u128,
    pub borrow_index: u128,
    pub last_update_timestamp: i64,
    pub ltv: u16,
    pub liquidation_threshold: u16,
    pub liquidation_bonus: u16,
    pub borrow_cap: u64,
    pub deposit_cap: u64,
    pub farm_address: Pubkey, 
    pub farm_balance: u64,
    pub is_active: u8,
    pub allow_deposits: u8,
    pub allow_borrows: u8,
    pub is_closed: u8,
}

impl StateDefinition for ReserveState {
    const LEN: usize = core::mem::size_of::<Self>();
    const SEED: &'static str = RESERVE_SEED;
}

impl DataLen for ReserveState {
    const LEN: usize = <Self as StateDefinition>::LEN;
}

impl ReserveState {
    pub fn new(
        lending_market: Pubkey,
        mint: Pubkey,
        ltv: u16,
        liquidation_threshold: u16,
        liquidation_bonus: u16,
        borrow_cap: u64,
        deposit_cap: u64,
    ) -> Self {
        Self {
            lending_market,
            mint,
            version: 0,
            available_liquidity: 0,
            total_supply: 0,
            total_borrows: 0,
            supply_index: 1_000_000_000_000_000_000, // 1.0 in 18 decimals
            borrow_index: 1_000_000_000_000_000_000, // 1.0 in 18 decimals
            last_update_timestamp: 0,
            ltv,
            liquidation_threshold,
            liquidation_bonus,
            borrow_cap,
            deposit_cap,
            farm_address: Pubkey::default(),
            farm_balance: 0,
            is_active: 1,
            allow_deposits: 1,
            allow_borrows: 1,
            is_closed: 0,
        }
    }

    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.is_active == 1
    }

    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        self.is_closed == 1
    }

    #[inline(always)]
    pub fn allows_deposits(&self) -> bool {
        self.allow_deposits == 1
    }

    #[inline(always)]
    pub fn allows_borrows(&self) -> bool {
        self.allow_borrows == 1
    }
}