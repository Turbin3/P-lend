use instructions::*;
use pinocchio::{ProgramResult, account_info::AccountInfo, pubkey::Pubkey};
use state::*;

pub mod instructions;
pub mod state;
pub mod helper;

pub use instructions::*;
pub use state::*;
pub use helper::*;

pinocchio_pubkey::declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");


pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);


    Ok(())
}