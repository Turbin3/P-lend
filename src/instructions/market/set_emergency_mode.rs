use crate::helper::{account_checks::check_signer, utils::DataLen};
use crate::state::LendingMarketState;
use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SetEmergencyModeIxData {
    pub enable: u8,
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

    let ix_data =
        bytemuck::from_bytes::<SetEmergencyModeIxData>(&data[..SetEmergencyModeIxData::LEN]);

    let data = &mut lending_market.try_borrow_mut_data()?;
    let lending_market_state = &mut bytemuck::from_bytes_mut::<LendingMarketState>(data);

    if authority.key() != &lending_market_state.lending_market_owner
        && authority.key() != &lending_market_state.risk_council
    {
        return Err(ProgramError::IllegalOwner);
    }

    lending_market_state.emergency_mode = ix_data.enable as u8;

    Ok(())
}
