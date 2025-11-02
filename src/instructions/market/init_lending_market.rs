use crate::helper::{
    account_checks::check_signer,
    account_init::{create_pda_account, StateDefinition},
    utils::{try_from_account_info_mut, DataLen},
};
use crate::state::LendingMarketState;
use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError, pubkey::Pubkey,
    sysvars::rent::Rent, ProgramResult,
};

use bytemuck::{Pod, Zeroable};


#[cfg(not(target_arch = "bpf"))]
fn find_program_address(
    seeds: &[&[u8]],
) -> Result<(Pubkey, u8), ProgramError> {
    use pinocchio::pubkey;

    let (derived, bump) = pubkey::find_program_address(seeds, &crate::ID);
    Ok((derived, bump))
}
#[repr(C)]
#[derive(Clone, Copy,Pod,Zeroable)]
pub struct InitLendingMarketIxData {
    pub lending_market_owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub risk_council: Pubkey,
}

impl DataLen for InitLendingMarketIxData {
    const LEN: usize = core::mem::size_of::<InitLendingMarketIxData>();
}

pub fn process_init_lending_market(
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

    let ix_data = bytemuck::from_bytes::<InitLendingMarketIxData>(&data[..InitLendingMarketIxData::LEN]);

    if lending_market_owner.key() != &ix_data.lending_market_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds = &[
        LendingMarketState::SEED.as_bytes(),
        ix_data.lending_market_owner.as_ref(),
    ];
    let (expected_market_key, bump) = find_program_address(seeds)?;

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
