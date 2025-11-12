use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

pub mod helper;
pub mod instructions;
pub mod state;

pub mod constants;
pub use helper::*;
pub use instructions::*;
pub use state::*;

pub use constants::*;

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if program_id != &ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (discriminant, payload) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    use instructions::{market, PlendInstructions::*};

    let instruction = instructions::PlendInstructions::try_from(*discriminant)?;

    match instruction {
        InitLendingMarket => {
            ensure_payload_len::<market::InitLendingMarketIxData>(payload)?;
            market::process_init_lending_market(accounts, payload)
        }
        SetEmergencyMode => {
            ensure_payload_len::<market::SetEmergencyModeIxData>(payload)?;
            market::process_set_emergency_mode(program_id, accounts, payload)
        }
        UpdateRiskCouncil => {
            ensure_payload_len::<market::UpdateRiskCouncilIxData>(payload)?;
            market::process_update_risk_council(program_id, accounts, payload)
        }
        UpdateLendingMarketOwner => {
            ensure_payload_len::<market::UpdateLendingMarketOwnerIxData>(payload)?;
            market::process_update_lending_market_owner(program_id, accounts, payload)
        }
    }
}

#[inline(always)]
fn ensure_payload_len<T: crate::helper::utils::DataLen>(
    payload: &[u8],
) -> Result<(), ProgramError> {
    if payload.len() == T::LEN {
        Ok(())
    } else {
        Err(ProgramError::InvalidInstructionData)
    }
}
