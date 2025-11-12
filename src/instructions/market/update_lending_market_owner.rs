use crate::helper::{
    account_checks::check_signer,
    utils::{DataLen},
};
use crate::state::LendingMarketState;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UpdateLendingMarketOwnerIxData {
    pub new_owner: Pubkey,
}

impl DataLen for UpdateLendingMarketOwnerIxData {
    const LEN: usize = core::mem::size_of::<UpdateLendingMarketOwnerIxData>();
}

pub fn process_update_lending_market_owner(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [current_owner, lending_market, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(current_owner)?;

    let ix_data = bytemuck::from_bytes::<UpdateLendingMarketOwnerIxData>(
        &data[..UpdateLendingMarketOwnerIxData::LEN],
    );

    let data = &mut lending_market.try_borrow_mut_data()?;
    let lending_market_state = &mut bytemuck::from_bytes_mut::<LendingMarketState>(data);

    if current_owner.key() != &lending_market_state.lending_market_owner {
        return Err(ProgramError::IllegalOwner);
    }

    lending_market_state.lending_market_owner = ix_data.new_owner;

    Ok(())
}
