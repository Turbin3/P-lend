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
pub struct EnableReserveIxData {}

impl DataLen for EnableReserveIxData {
    const LEN: usize = 0;
}

pub fn process_enable_reserve(
    program_id: &pinocchio::pubkey::Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let [authority, reserve, lending_market, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(authority)?;

    // Load lending market state
    let lending_market_data = lending_market.try_borrow_data()?;
    let lending_market_state = bytemuck::from_bytes::<LendingMarketState>(&lending_market_data);

    // Verify authority is either lending market owner or risk council
    let authority_key = authority.key();
    let is_market_owner = authority_key.as_ref() == &lending_market_state.lending_market_owner;
    let is_risk_council = authority_key.as_ref() == &lending_market_state.risk_council;

    if !is_market_owner && !is_risk_council {
        return Err(ProgramError::IllegalOwner);
    }

    // Verify reserve account ownership
    if reserve.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load and update reserve state
    let reserve_data = &mut reserve.try_borrow_mut_data()?;
    let reserve_state = bytemuck::from_bytes_mut::<ReserveState>(reserve_data);


    // Verify reserve belongs to this lending market
    if reserve_state.lending_market != *lending_market.key() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if reserve is closed
    if reserve_state.allow_deposits == 0 && reserve_state.allow_borrows == 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Enable the reserve
    reserve_state.is_active = 1;

    Ok(())
}
