use crate::helper::{
    account_checks::check_signer,
    utils::{load_ix_data, try_from_account_info_mut, DataLen},
};
use crate::state::LendingMarketState;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UpdateRiskCouncilIxData {
    pub new_risk_council: Pubkey,
}

impl DataLen for UpdateRiskCouncilIxData {
    const LEN: usize = core::mem::size_of::<UpdateRiskCouncilIxData>();
}

pub fn process_update_risk_council(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [owner, lending_market, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(owner)?;

    let ix_data = unsafe { load_ix_data::<UpdateRiskCouncilIxData>(data)? };

    unsafe {
        let state = try_from_account_info_mut::<LendingMarketState>(lending_market)?;

        if owner.key() != &state.lending_market_owner {
            return Err(ProgramError::IllegalOwner);
        }

        state.risk_council = ix_data.new_risk_council;
    }

    Ok(())
}
