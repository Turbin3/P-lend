use crate::state::{LendingMarketState, ReserveState};
use crate::helper::{
    account_checks::check_signer,
    utils::DataLen,
};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    ProgramResult,
};
use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CloseReserveIxData {}

impl DataLen for CloseReserveIxData {
    const LEN: usize = 0;
}

pub fn process_close_reserve(
    program_id: &pinocchio::pubkey::Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let [lending_market_owner, reserve, lending_market, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(lending_market_owner)?;

    // Load lending market state
    let lending_market_data = lending_market.try_borrow_data()?;
    let lending_market_state = bytemuck::from_bytes::<LendingMarketState>(&lending_market_data);

    // Verify authority is lending market owner (only owner can close reserves permanently)
    if lending_market_owner.key().as_ref() != &lending_market_state.lending_market_owner {
        return Err(ProgramError::IllegalOwner);
    }

    // Verify reserve account ownership
    if reserve.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load and update reserve state
    let reserve_data = &mut reserve.try_borrow_mut_data()?;
    let reserve_state = bytemuck::from_bytes_mut::<ReserveState>(reserve_data);

    // Check if already closed
    if reserve_state.is_closed() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Verify reserve belongs to this lending market
    if reserve_state.lending_market != *lending_market.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify reserve has no outstanding liquidity or borrows
    if reserve_state.available_liquidity != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    if reserve_state.total_borrows != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Close the reserve permanently
    reserve_state.is_closed = 1;
    reserve_state.is_active = 0;
    reserve_state.allow_deposits = 0;
    reserve_state.allow_borrows = 0;

    Ok(())
}
