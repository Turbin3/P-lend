use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

pub mod helper;
pub mod instructions;
pub mod state;

pub use helper::*;
pub use instructions::*;
pub use state::*;

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

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

    match LendingMarketInstruction::try_from(*discriminant)? {
        LendingMarketInstruction::InitLendingMarket => {
            use crate::instructions::init_lending_market::InitLendingMarketIxData;

            if payload.len() != InitLendingMarketIxData::LEN {
                return Err(ProgramError::InvalidInstructionData);
            }

            process_init_lending_market(program_id, accounts, payload)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
