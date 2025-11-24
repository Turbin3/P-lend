use crate::state::ReserveState;
use crate::constants::RESERVE_VAULT_SEED;
use crate::helper::{
    account_checks::check_signer,
    utils::DataLen,
};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use bytemuck::{Pod, Zeroable};

#[cfg(not(target_arch = "bpf"))]
fn find_program_address(seeds: &[&[u8]]) -> Result<(Pubkey, u8), ProgramError> {
    let (derived, bump) = pinocchio::pubkey::find_program_address(seeds, &crate::ID);
    Ok((derived, bump))
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SupplyLiquidityIxData {
    pub amount: u64,
}

impl DataLen for SupplyLiquidityIxData {
    const LEN: usize = core::mem::size_of::<SupplyLiquidityIxData>();
}

pub fn process_supply_liquidity(
    program_id: &pinocchio::pubkey::Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        user,
        user_token_account,
        reserve,
        reserve_vault,
        _token_program,
        _remaining @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(user)?;

    // Parse instruction data
    let ix_data = bytemuck::from_bytes::<SupplyLiquidityIxData>(&data[..SupplyLiquidityIxData::LEN]);
    let amount = ix_data.amount;

    if amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Verify reserve account ownership
    if reserve.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load reserve state
    let reserve_data = &mut reserve.try_borrow_mut_data()?;
    let reserve_state = bytemuck::from_bytes_mut::<ReserveState>(reserve_data);

    // Check if reserve is active
    if !reserve_state.is_active() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if reserve is closed
    if reserve_state.is_closed() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if deposits are allowed
    if !reserve_state.allows_deposits() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check deposit cap
    let new_total_supply = reserve_state.total_supply
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if reserve_state.deposit_cap > 0 && new_total_supply > reserve_state.deposit_cap {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Validate reserve vault PDA
    let vault_seeds = &[
        RESERVE_VAULT_SEED.as_bytes(),
        reserve.key().as_ref(),
    ];
    let (expected_vault_key, _) = find_program_address(vault_seeds)?;

    if expected_vault_key != *reserve_vault.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // Transfer tokens from user to reserve vault
   pinocchio_token::instructions::Transfer {
        from: user_token_account,
        to: reserve_vault,
        authority: user,
        amount,
    }.invoke()?;

    // Update reserve state
    reserve_state.available_liquidity = reserve_state.available_liquidity
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    reserve_state.total_supply = new_total_supply;

    Ok(())
}
