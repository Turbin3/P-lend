use pinocchio::{program_error::ProgramError, pubkey::Pubkey};
use crate::helper::utils::DataLen;
use pinocchio::account_info::AccountInfo;
use pinocchio::ProgramResult;


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
    let [lending_market_owner, quote_currency, risk_council, lending_market, _] = accounts
    else {
        return Err(ProgramError::InvalidInstructionData);
    };



    Ok(())
}