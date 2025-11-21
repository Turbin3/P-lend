use crate::helper::{
    account_checks::check_signer,
    account_init::{create_pda_account, StateDefinition},
    utils::DataLen,
};
use crate::state::{ObligationCollateral, ObligationLiquidity, ObligationState};
use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent},
    ProgramResult,
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
pub struct InitObligationIxData {
    pub id: u8,
}

impl DataLen for InitObligationIxData {
    const LEN: usize = core::mem::size_of::<InitObligationIxData>();
}

pub fn process_init_lending_market(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [obligation_owner, obligation, lending_market, clock, rent, _system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Accounts Checks
    check_signer(obligation_owner)?;

    if !obligation.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let ix_data = bytemuck::from_bytes::<InitObligationIxData>(&data[..InitObligationIxData::LEN]);

    let seeds = &[
        ObligationState::SEED.as_bytes(),
        &ix_data.id.to_le_bytes(),
        obligation_owner.key(),
        lending_market.key(),
    ];
    let (expected_market_key, bump) = find_program_address(seeds)?;

    if expected_market_key != *obligation.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::from_account_info(rent)?;
    let id_binding = [ix_data.id];
    let bump_binding = [bump];
    let signer_seeds = [
        Seed::from(ObligationState::SEED.as_bytes()),
        Seed::from(&id_binding),
        Seed::from(obligation_owner.key().as_ref()),
        Seed::from(lending_market.key().as_ref()),
        Seed::from(&bump_binding),
    ];

    create_pda_account::<ObligationState>(obligation_owner, obligation, &signer_seeds, &rent)?;

    let mut obligation_data = obligation.try_borrow_mut_data()?;
    let obligation =
        bytemuck::from_bytes_mut::<ObligationState>(&mut obligation_data[..ObligationState::LEN]);

    let clock = Clock::from_account_info(clock)?;

    obligation.lending_market = *lending_market.key();
    obligation.owner = *obligation_owner.key();
    obligation.deposits = [ObligationCollateral::default(); 8];
    obligation.borrows = [ObligationLiquidity::default(); 5];
    obligation.health_factor = 0;
    obligation.liquidation_threshold = 0;
    obligation.last_update_slot = clock.slot;

    Ok(())
}
