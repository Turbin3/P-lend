use pinocchio::{
    account_info::AccountInfo,
    instruction::Seed,
    program_error::ProgramError,
    pubkey,
    sysvars::{clock::Clock, rent::Rent},
    ProgramResult,
};

use crate::{
    create_pda_account, helper::account_checks::check_signer, ObligationCollateral,
    ObligationLiquidity, ObligationState, StateDefinition,
};

pub struct InitObligationAccounts<'a> {
    pub obligation_owner: &'a AccountInfo,
    pub obligation: &'a AccountInfo,
    pub lending_market: &'a AccountInfo,
    pub clock: &'a AccountInfo,
    pub rent: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitObligationAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [obligation_owner, obligation, lending_market, clock, rent, system_program] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Accounts Checks
        check_signer(obligation_owner)?;

        Ok(Self {
            obligation_owner,
            obligation,
            lending_market,
            clock,
            rent,
            system_program,
        })
    }
}

pub struct InitObligationIxData {
    pub id: u8,
}

impl<'a> TryFrom<&'a [u8]> for InitObligationIxData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u8>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let id = u8::from_le_bytes(data[0..1].try_into().unwrap());

        Ok(Self { id })
    }
}

pub struct InitObligation<'a> {
    pub accounts: InitObligationAccounts<'a>,
    pub instruction_data: InitObligationIxData,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for InitObligation<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = InitObligationAccounts::try_from(accounts)?;
        let instruction_data = InitObligationIxData::try_from(data)?;

        // Initialize the Accounts needed
        let (_, bump) = pubkey::find_program_address(
            &[
                ObligationState::SEED.as_bytes(),
                &instruction_data.id.to_le_bytes(),
                accounts.obligation_owner.key(),
                accounts.lending_market.key(),
            ],
            &crate::ID,
        );

        let rent = Rent::from_account_info(accounts.rent)?;
        let id_binding = [instruction_data.id];
        let bump_binding = [bump];
        let signer_seeds = [
            Seed::from(ObligationState::SEED.as_bytes()),
            Seed::from(&id_binding),
            Seed::from(accounts.obligation_owner.key().as_ref()),
            Seed::from(accounts.lending_market.key().as_ref()),
            Seed::from(&bump_binding),
        ];

        create_pda_account::<ObligationState>(
            accounts.obligation_owner,
            accounts.obligation,
            &signer_seeds,
            &rent,
        )?;

        Ok(Self {
            accounts,
            instruction_data,
            bump,
        })
    }
}

impl<'a> InitObligation<'a> {
    pub fn process(&mut self) -> ProgramResult {
        let mut obligation_data = self.accounts.obligation.try_borrow_mut_data()?;
        let obligation = bytemuck::from_bytes_mut::<ObligationState>(
            &mut obligation_data[..ObligationState::LEN],
        );

        let clock = Clock::from_account_info(self.accounts.clock)?;

        obligation.lending_market = *self.accounts.lending_market.key();
        obligation.owner = *self.accounts.obligation_owner.key();
        obligation.deposits = [ObligationCollateral::default(); 8];
        obligation.borrows = [ObligationLiquidity::default(); 5];
        obligation.health_factor = 0;
        obligation.liquidation_threshold = 0;
        obligation.last_update_slot = clock.slot;

        Ok(())
    }
}
