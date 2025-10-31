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
pub struct SetEmergencyModeIxData {
    pub enable: bool,
}

impl DataLen for SetEmergencyModeIxData {
    const LEN: usize = core::mem::size_of::<SetEmergencyModeIxData>();
}

pub fn process_set_emergency_mode(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [authority, lending_market, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(authority)?;

    let ix_data = unsafe { load_ix_data::<SetEmergencyModeIxData>(data)? };

    unsafe {
        let state = try_from_account_info_mut::<LendingMarketState>(lending_market)?;

        let key = authority.key();
        if key != &state.lending_market_owner && key != &state.risk_council {
            return Err(ProgramError::IllegalOwner);
        }

        state.emergency_mode = ix_data.enable;
    }

    Ok(())
}
