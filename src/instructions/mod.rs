pub mod market;
pub mod reserve;

pub use market::*;
pub use reserve::*;
use pinocchio::program_error::ProgramError;

pub enum PlendInstructions {
    InitLendingMarket = 0,
    UpdateLendingMarketOwner = 1,
    SetEmergencyMode = 2,
    UpdateRiskCouncil = 3,
    InitReserve = 4,
}

impl TryFrom<u8> for PlendInstructions {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PlendInstructions::InitLendingMarket),
            1 => Ok(PlendInstructions::UpdateLendingMarketOwner),
            2 => Ok(PlendInstructions::SetEmergencyMode),
            3 => Ok(PlendInstructions::UpdateRiskCouncil),
            4 => Ok(PlendInstructions::InitReserve),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
