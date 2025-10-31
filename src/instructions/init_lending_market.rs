use crate::helper::{
    account_checks::check_signer,
    account_init::{create_pda_account, StateDefinition},
    utils::{load_ix_data, try_from_account_info_mut, DataLen},
};
use crate::state::LendingMarketState;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::rent::Rent,
    ProgramResult,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitLendingMarketIxData {
    pub lending_market_owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub risk_council: Pubkey,
}

impl DataLen for InitLendingMarketIxData {
    const LEN: usize = core::mem::size_of::<InitLendingMarketIxData>();
}

pub fn process_init_lending_market(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [lending_market_owner, lending_market, rent_sysvar, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(lending_market_owner)?;

    if !lending_market.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let ix_data = unsafe { load_ix_data::<InitLendingMarketIxData>(data)? };

    if lending_market_owner.key() != &ix_data.lending_market_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds = &[
        LendingMarketState::SEED.as_bytes(),
        ix_data.lending_market_owner.as_ref(),
    ];
    let (expected_market_key, bump) = pubkey::find_program_address(seeds, program_id);

    if expected_market_key != *lending_market.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::from_account_info(rent_sysvar)?;
    let bump_bytes = [bump];
    let signer_seeds = [
        Seed::from(LendingMarketState::SEED.as_bytes()),
        Seed::from(ix_data.lending_market_owner.as_ref()),
        Seed::from(&bump_bytes[..]),
    ];

    create_pda_account::<LendingMarketState>(
        lending_market_owner,
        lending_market,
        &signer_seeds,
        &rent,
    )?;

    unsafe {
        let state = try_from_account_info_mut::<LendingMarketState>(lending_market)?;
        *state = LendingMarketState::new(
            ix_data.lending_market_owner,
            ix_data.quote_currency,
            ix_data.risk_council,
        );
    }

    Ok(())
}
