pub mod init_lending_market;
pub mod set_emergency_mode;
pub mod update_lending_market_owner;
pub mod update_risk_council;

pub use init_lending_market::*;
use pinocchio::program_error::ProgramError;
pub use set_emergency_mode::*;
pub use update_lending_market_owner::*;
pub use update_risk_council::*;

pub enum LendingMarketInstruction {
    InitLendingMarket = 0,
    UpdateLendingMarketOwner = 1,
    SetEmergencyMode = 2,
    UpdateRiskCouncil = 3,
}

impl TryFrom<u8> for LendingMarketInstruction {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LendingMarketInstruction::InitLendingMarket),
            1 => Ok(LendingMarketInstruction::UpdateLendingMarketOwner),
            2 => Ok(LendingMarketInstruction::SetEmergencyMode),
            3 => Ok(LendingMarketInstruction::UpdateRiskCouncil),
        }
    }
}
