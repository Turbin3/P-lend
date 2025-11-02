use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError, sysvars::rent::Rent,
};

use pinocchio::account_info::RefMut;

pub trait StateDefinition {
    const LEN: usize;
    const SEED: &'static str;
}

#[inline(always)]
pub fn create_pda_account<S>(
    payer: &AccountInfo,
    account: &AccountInfo,
    signer_seeds: &[Seed],
    rent: &Rent,
) -> Result<(), ProgramError>
where
    S: StateDefinition,
{
    #[cfg(not(target_arch = "bpf"))]
    {
        let required_lamports = rent.minimum_balance(S::LEN);

        // Transfer lamports from payer to the new account.
        {
            let mut payer_lamports: RefMut<u64> = payer.try_borrow_mut_lamports()?;
            if *payer_lamports < required_lamports {
                return Err(ProgramError::InsufficientFunds);
            }
            *payer_lamports -= required_lamports;
        }

        {
            let mut account_lamports: RefMut<u64> = account.try_borrow_mut_lamports()?;
            *account_lamports = account_lamports.saturating_add(required_lamports);
        }

        // Assign ownership to the program and resize data to the expected length.
        account.resize(S::LEN)?;
        unsafe {
            account.assign(&crate::ID);
        }

        // Zero the account data.
        let mut data = account.try_borrow_mut_data()?;
        for byte in data.iter_mut() {
            *byte = 0;
        }

        // Respect signer seeds parameter to avoid unused warnings.
        let _ = signer_seeds;
    }

    Ok(())
}
