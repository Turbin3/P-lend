use crate::helper::{
    account_checks::check_signer,
    utils::{ DataLen},
};
use crate::state::LendingMarketState;
use bytemuck::{Pod, Zeroable};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
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

    let ix_data =
        bytemuck::from_bytes::<UpdateRiskCouncilIxData>(&data[..UpdateRiskCouncilIxData::LEN]);

    let data = &mut lending_market.try_borrow_mut_data()?;
    let lending_market_state = &mut bytemuck::from_bytes_mut::<LendingMarketState>(data);

    if owner.key() != &lending_market_state.lending_market_owner {
        return Err(ProgramError::IllegalOwner);
    }

    lending_market_state.risk_council = ix_data.new_risk_council;

    Ok(())
}
