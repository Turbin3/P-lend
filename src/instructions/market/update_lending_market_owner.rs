use crate::helper::{
    account_checks::check_signer,
    utils::{try_from_account_info_mut, DataLen},
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

    unsafe {
        let state = try_from_account_info_mut::<LendingMarketState>(lending_market)?;

        if current_owner.key() != &state.lending_market_owner {
            return Err(ProgramError::IllegalOwner);
        }

        state.lending_market_owner = ix_data.new_owner;
    }

    Ok(())
}
