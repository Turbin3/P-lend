mod common;

use common::initialize_lending_market;
use plend::instructions::{
    set_emergency_mode::SetEmergencyModeIxData,
    update_lending_market_owner::UpdateLendingMarketOwnerIxData,
    update_risk_council::UpdateRiskCouncilIxData, PlendInstructions,
};
use solana_instruction::{account_meta::AccountMeta, error::InstructionError, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction_error::TransactionError;

fn encode_instruction<T: plend::helper::utils::DataLen>(
    discriminant: PlendInstructions,
    payload: &T,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + T::LEN);
    data.push(discriminant as u8);
    data.extend_from_slice(common::serialize_struct(payload));
    data
}

#[test]
fn test_init_lending_market() {
    let ctx = initialize_lending_market();
    let state = ctx.market_state();

    assert_eq!(state.lending_market_owner, ctx.owner_pubkey());
    assert_eq!(state.risk_council, ctx.risk_council_pubkey());
    assert_eq!(state.emergency_mode, 0);
}

#[test]
#[ignore]
fn test_set_emergency_mode() {
    let mut ctx = initialize_lending_market();
    let instruction = ctx.build_set_emergency_mode_instruction(1);

    ctx.send_instruction(
        vec![
            ctx.fee_payer.insecure_clone(),
            ctx.risk_council.insecure_clone(),
        ],
        instruction,
    )
    .unwrap();

    let state = ctx.market_state();
    assert_eq!(state.emergency_mode, 1);
}

#[test]
#[ignore]
fn test_set_emergency_mode_requires_authority() {
    let mut ctx = initialize_lending_market();
    let unauthorized = Keypair::new();
    let unauthorized_pubkey = unauthorized.pubkey();
    ctx.airdrop(&unauthorized_pubkey, 1_000_000_000);

    let data = encode_instruction(
        PlendInstructions::SetEmergencyMode,
        &SetEmergencyModeIxData { enable: 1 },
    );
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(unauthorized_pubkey, true),
            AccountMeta::new(ctx.market_pubkey, false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(
            vec![
                ctx.fee_payer.insecure_clone(),
                unauthorized.insecure_clone(),
            ],
            instruction,
        )
        .expect_err("unauthorized authority should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::ILLEGAL_OWNER as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
#[ignore]
fn test_update_risk_council() {
    let mut ctx = initialize_lending_market();
    let new_risk = [15u8; 32];
    let instruction = ctx.build_update_risk_council_instruction(new_risk);

    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .unwrap();

    let state = ctx.market_state();
    assert_eq!(state.risk_council, new_risk);
}

#[test]
#[ignore]
fn test_update_risk_council_requires_owner() {
    let mut ctx = initialize_lending_market();
    let new_risk = [21u8; 32];
    let unauthorized = Keypair::new();
    let unauthorized_pubkey = unauthorized.pubkey();
    ctx.airdrop(&unauthorized_pubkey, 1_000_000_000);

    let data = encode_instruction(
        PlendInstructions::UpdateRiskCouncil,
        &UpdateRiskCouncilIxData {
            new_risk_council: new_risk,
        },
    );
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(unauthorized_pubkey, true),
            AccountMeta::new(ctx.market_pubkey, false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(
            vec![
                ctx.fee_payer.insecure_clone(),
                unauthorized.insecure_clone(),
            ],
            instruction,
        )
        .expect_err("only owner should be allowed to update risk council");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::ILLEGAL_OWNER as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
#[ignore]
fn test_update_lending_market_owner() {
    let mut ctx = initialize_lending_market();
    let new_owner = [33u8; 32];
    let instruction = ctx.build_update_owner_instruction(new_owner);

    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .unwrap();

    let state = ctx.market_state();
    assert_eq!(state.lending_market_owner, new_owner);
}

#[test]
#[ignore]
fn test_update_lending_market_owner_requires_owner() {
    let mut ctx = initialize_lending_market();
    let new_owner = [44u8; 32];
    let unauthorized = Keypair::new();
    let unauthorized_pubkey = unauthorized.pubkey();
    ctx.airdrop(&unauthorized_pubkey, 1_000_000_000);

    let data = encode_instruction(
        PlendInstructions::UpdateLendingMarketOwner,
        &UpdateLendingMarketOwnerIxData { new_owner },
    );
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(unauthorized_pubkey, true),
            AccountMeta::new(ctx.market_pubkey, false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(
            vec![
                ctx.fee_payer.insecure_clone(),
                unauthorized.insecure_clone(),
            ],
            instruction,
        )
        .expect_err("only current owner may transfer ownership");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::ILLEGAL_OWNER as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}
