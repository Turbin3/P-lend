use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

#[inline(always)]
pub fn check_signer(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}
