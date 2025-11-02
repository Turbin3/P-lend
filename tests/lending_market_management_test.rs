mod common;

use common::{initialize_lending_market, serialize_struct};
use p_lend::instructions::{
    set_emergency_mode::{process_set_emergency_mode, SetEmergencyModeIxData},
    update_lending_market_owner::{
        process_update_lending_market_owner, UpdateLendingMarketOwnerIxData,
    },
    update_risk_council::{process_update_risk_council, UpdateRiskCouncilIxData},
};
use solana_pubkey::Pubkey;

#[test]
fn test_init_lending_market() {
    let ctx = initialize_lending_market();
    let state = ctx.market_state();

    assert_eq!(state.lending_market_owner, *ctx.owner_pubkey());
    assert_eq!(state.risk_council, ctx.risk_council_pubkey);
    assert_eq!(state.emergency_mode, 0);
}

#[test]
fn test_set_emergency_mode() {
    let ctx = initialize_lending_market();

    let ix_data = SetEmergencyModeIxData { enable: 1 };
    let accounts = [ctx.risk_council_account.info(), ctx.market.info()];

    process_set_emergency_mode(&ctx.program_id, &accounts, serialize_struct(&ix_data)).unwrap();

    let state = ctx.market_state();
    assert_eq!(state.emergency_mode, 1);
}

#[test]
fn test_set_emergency_mode_requires_authority() {
    let ctx = initialize_lending_market();

    let ix_data = SetEmergencyModeIxData { enable: 1 };
    let unauthorized =
        common::TestAccount::new(Pubkey::new_from_array([9u8; 32]), common::system_program(), 0, 0, true, false);
    let accounts = [unauthorized.info(), ctx.market.info()];

    let err = process_set_emergency_mode(&ctx.program_id, &accounts, serialize_struct(&ix_data))
        .expect_err("unauthorized should fail");
    assert_eq!(err, pinocchio::program_error::ProgramError::IllegalOwner);
}

#[test]
fn test_update_risk_council() {
    let ctx = initialize_lending_market();
    let new_risk = [15u8; 32];

    let ix_data = UpdateRiskCouncilIxData {
        new_risk_council: new_risk,
    };
    let accounts = [ctx.owner_account_info(), ctx.market.info()];

    process_update_risk_council(&ctx.program_id, &accounts, serialize_struct(&ix_data)).unwrap();

    let state = ctx.market_state();
    assert_eq!(state.risk_council, new_risk);
}

#[test]
fn test_update_risk_council_requires_owner() {
    let ctx = initialize_lending_market();
    let new_risk = [21u8; 32];

    let ix_data = UpdateRiskCouncilIxData {
        new_risk_council: new_risk,
    };
    let unauthorized =
        common::TestAccount::new(Pubkey::new_from_array([7u8; 32]), common::system_program(), 0, 0, true, false);
    let accounts = [unauthorized.info(), ctx.market.info()];

    let err = process_update_risk_council(&ctx.program_id, &accounts, serialize_struct(&ix_data))
        .expect_err("only owner may update risk council");
    assert_eq!(err, pinocchio::program_error::ProgramError::IllegalOwner);
}

#[test]
fn test_update_lending_market_owner() {
    let ctx = initialize_lending_market();
    let new_owner = [33u8; 32];

    let ix_data = UpdateLendingMarketOwnerIxData { new_owner };
    let accounts = [ctx.owner_account_info(), ctx.market.info()];

    process_update_lending_market_owner(&ctx.program_id, &accounts, serialize_struct(&ix_data))
        .unwrap();

    let state = ctx.market_state();
    assert_eq!(state.lending_market_owner, new_owner);
}

#[test]
fn test_update_lending_market_owner_requires_owner() {
    let ctx = initialize_lending_market();
    let new_owner = [44u8; 32];

    let ix_data = UpdateLendingMarketOwnerIxData { new_owner };
    let unauthorized =
        common::TestAccount::new(Pubkey::new_from_array([8u8; 32]), common::system_program(), 0, 0, true, false);
    let accounts = [unauthorized.info(), ctx.market.info()];

    let err =
        process_update_lending_market_owner(&ctx.program_id, &accounts, serialize_struct(&ix_data))
            .expect_err("only current owner may transfer ownership");
    assert_eq!(err, pinocchio::program_error::ProgramError::IllegalOwner);
}
