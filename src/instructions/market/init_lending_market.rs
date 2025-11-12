use crate::state::LendingMarketState;
use crate::{
    helper::{
        account_checks::check_signer,
        account_init::{create_pda_account, StateDefinition},
        utils::DataLen,
    },
    LENDING_MARKET_SEED,
};
use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError, pubkey::Pubkey,
    sysvars::rent::Rent, ProgramResult,
};

use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey;

#[cfg(not(target_arch = "bpf"))]
fn find_program_address(seeds: &[&[u8]]) -> Result<(Pubkey, u8), ProgramError> {
    let (derived, bump) = pubkey::find_program_address(seeds, &crate::ID);
    Ok((derived, bump))
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitLendingMarketIxData {
    pub lending_market_owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub risk_council: Pubkey,
}

impl DataLen for InitLendingMarketIxData {
    const LEN: usize = core::mem::size_of::<InitLendingMarketIxData>();
}

pub fn process_init_lending_market(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [lending_market_owner, lending_market, rent_sysvar, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(lending_market_owner)?;

    if !lending_market.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let ix_data =
        bytemuck::from_bytes::<InitLendingMarketIxData>(&data[..InitLendingMarketIxData::LEN]);

    if lending_market_owner.key() != &ix_data.lending_market_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds = &[
        LENDING_MARKET_SEED.as_bytes(),
        ix_data.lending_market_owner.as_ref(),
    ];
    let (expected_market_key, bump) = find_program_address(seeds)?;

    if expected_market_key != *lending_market.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::from_account_info(rent_sysvar)?;
    let bump_bytes = [bump];
    let lending_market_seeds = [
        Seed::from(LendingMarketState::SEED.as_bytes()),
        Seed::from(ix_data.lending_market_owner.as_ref()),
        Seed::from(&bump_bytes[..]),
    ];

    create_pda_account::<LendingMarketState>(
        lending_market_owner,
        lending_market,
        &lending_market_seeds,
        &rent,
    )?;

    let data = &mut lending_market.try_borrow_mut_data()?;

    let lending_market_state = &mut bytemuck::from_bytes_mut::<LendingMarketState>(data);

    lending_market_state.emergency_mode = 0;
    lending_market_state.lending_market_owner = ix_data.lending_market_owner;
    lending_market_state.quote_currency = ix_data.quote_currency;
    lending_market_state.version = 0;
    lending_market_state.risk_council = ix_data.risk_council;

    Ok(())
}
