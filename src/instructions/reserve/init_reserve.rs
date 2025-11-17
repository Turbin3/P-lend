use crate::state::{LendingMarketState, ReserveState};
use crate::{
    helper::{
        account_checks::check_signer,
        account_init::{create_pda_account, StateDefinition},
        utils::DataLen,
    },
    constants::RESERVE_SEED,
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

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InitReserveIxData {
    pub ltv: u16,
    pub liquidation_threshold: u16,
    pub liquidation_bonus: u16,
    pub borrow_cap: u64,
    pub deposit_cap: u64,
}

impl DataLen for InitReserveIxData {
    const LEN: usize = core::mem::size_of::<InitReserveIxData>();
}

pub fn process_init_reserve(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [lending_market_owner, reserve, lending_market, mint, rent_sysvar, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    check_signer(lending_market_owner)?;
    if !reserve.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    let ix_data = bytemuck::from_bytes::<InitReserveIxData>(&data[..InitReserveIxData::LEN]);
    let lending_market_data = lending_market.try_borrow_data()?;
    let lending_market_state = bytemuck::from_bytes::<LendingMarketState>(&lending_market_data);
    if lending_market_owner.key() != &lending_market_state.lending_market_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate reserve PDA
    let seeds = &[
        RESERVE_SEED.as_bytes(),
        lending_market.key().as_ref(),
        mint.key().as_ref(),
    ];
    let (expected_reserve_key, bump) = find_program_address(seeds)?;

    if expected_reserve_key != *reserve.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // Validate input parameters
    if ix_data.ltv > 10000 { 
        return Err(ProgramError::InvalidInstructionData);
    }
    
    if ix_data.liquidation_threshold > 10000 { 
        return Err(ProgramError::InvalidInstructionData);
    }
    
    if ix_data.liquidation_bonus > 2000 { 
        return Err(ProgramError::InvalidInstructionData);
    }

    if ix_data.ltv >= ix_data.liquidation_threshold {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Create the reserve PDA account
    let rent = Rent::from_account_info(rent_sysvar)?;
    let bump_bytes = [bump];
    let reserve_seeds = [
        Seed::from(ReserveState::SEED.as_bytes()),
        Seed::from(lending_market.key().as_ref()),
        Seed::from(mint.key().as_ref()),
        Seed::from(&bump_bytes[..]),
    ];

    create_pda_account::<ReserveState>(
        lending_market_owner,
        reserve,
        &reserve_seeds,
        &rent,
    )?;

    // Initialize the reserve state
    let reserve_data = &mut reserve.try_borrow_mut_data()?;
    let reserve_state = &mut bytemuck::from_bytes_mut::<ReserveState>(reserve_data);

    **reserve_state = ReserveState::new(
        *lending_market.key(),
        *mint.key(),
        ix_data.ltv,
        ix_data.liquidation_threshold,
        ix_data.liquidation_bonus,
        ix_data.borrow_cap,
        ix_data.deposit_cap,
    );

    Ok(())
}