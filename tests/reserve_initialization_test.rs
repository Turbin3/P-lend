mod common;

use common::initialize_lending_market;
use plend::{
    constants::RESERVE_SEED,
    instructions::{init_reserve::InitReserveIxData, PlendInstructions},
};
use solana_instruction::{account_meta::AccountMeta, error::InstructionError, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction_error::TransactionError;
use pinocchio::sysvars::rent::RENT_ID;
use pinocchio_system::ID as SYSTEM_PROGRAM_ID;

fn encode_instruction<T: plend::helper::utils::DataLen>(
    discriminant: PlendInstructions,
    payload: &T,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(1 + T::LEN);
    data.push(discriminant as u8);
    data.extend_from_slice(common::serialize_struct(payload));
    data
}

fn create_test_mint() -> Pubkey {
    // Create a fake mint pubkey for testing
    Pubkey::new_unique()
}

#[test]
fn test_init_reserve_success() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    
    let (instruction, reserve_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        8000,  // 80% LTV
        8500,  // 85% liquidation threshold
        500,   // 5% liquidation bonus
        1_000_000_000, // 1B borrow cap
        2_000_000_000, // 2B deposit cap
    );

    // Execute the instruction
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect("Reserve initialization should succeed");

    // Verify the reserve state  
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    
    // Copy packed fields to avoid alignment issues
    let lending_market = reserve_state.lending_market;
    let mint_bytes = reserve_state.mint;
    let version = reserve_state.version;
    let available_liquidity = reserve_state.available_liquidity;
    let total_supply = reserve_state.total_supply;
    let total_borrows = reserve_state.total_borrows;
    let supply_index = reserve_state.supply_index;
    let borrow_index = reserve_state.borrow_index;
    let ltv = reserve_state.ltv;
    let liquidation_threshold = reserve_state.liquidation_threshold;
    let liquidation_bonus = reserve_state.liquidation_bonus;
    let borrow_cap = reserve_state.borrow_cap;
    let deposit_cap = reserve_state.deposit_cap;
    let farm_address = reserve_state.farm_address;
    let farm_balance = reserve_state.farm_balance;
    let is_active = reserve_state.is_active;
    let allow_deposits = reserve_state.allow_deposits;
    let allow_borrows = reserve_state.allow_borrows;
    
    assert_eq!(lending_market, ctx.market_pubkey.to_bytes());
    assert_eq!(mint_bytes, mint.to_bytes());
    assert_eq!(version, 0);
    assert_eq!(available_liquidity, 0);
    assert_eq!(total_supply, 0);
    assert_eq!(total_borrows, 0);
    assert_eq!(supply_index, 1_000_000_000_000_000_000); // 1.0 in 18 decimals
    assert_eq!(borrow_index, 1_000_000_000_000_000_000); // 1.0 in 18 decimals
    assert_eq!(ltv, 8000);
    assert_eq!(liquidation_threshold, 8500);
    assert_eq!(liquidation_bonus, 500);
    assert_eq!(borrow_cap, 1_000_000_000);
    assert_eq!(deposit_cap, 2_000_000_000);
    assert_eq!(farm_address, [0u8; 32]); // Zero pubkey
    assert_eq!(farm_balance, 0);
    assert_eq!(is_active, 1);
    assert_eq!(allow_deposits, 1);
    assert_eq!(allow_borrows, 1);
}

#[test]
fn test_init_reserve_with_minimal_params() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    
    let (instruction, reserve_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        0,    // 0% LTV
        100,  // 1% liquidation threshold
        0,    // 0% liquidation bonus
        0,    // 0 borrow cap
        1,    // 1 deposit cap
    );

    // Execute the instruction
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect("Reserve initialization with minimal params should succeed");

    // Verify the reserve state
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    
    // Copy packed fields to avoid alignment issues
    let ltv = reserve_state.ltv;
    let liquidation_threshold = reserve_state.liquidation_threshold;
    let liquidation_bonus = reserve_state.liquidation_bonus;
    let borrow_cap = reserve_state.borrow_cap;
    let deposit_cap = reserve_state.deposit_cap;
    
    assert_eq!(ltv, 0);
    assert_eq!(liquidation_threshold, 100);
    assert_eq!(liquidation_bonus, 0);
    assert_eq!(borrow_cap, 0);
    assert_eq!(deposit_cap, 1);
}

#[test]
fn test_init_reserve_with_maximum_params() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    
    let (instruction, reserve_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        9999,  // 99.99% LTV
        10000, // 100% liquidation threshold
        2000,  // 20% liquidation bonus (max allowed)
        u64::MAX, // Max borrow cap
        u64::MAX, // Max deposit cap
    );

    // Execute the instruction
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect("Reserve initialization with maximum params should succeed");

    // Verify the reserve state
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    
    // Copy packed fields to avoid alignment issues
    let ltv = reserve_state.ltv;
    let liquidation_threshold = reserve_state.liquidation_threshold;
    let liquidation_bonus = reserve_state.liquidation_bonus;
    let borrow_cap = reserve_state.borrow_cap;
    let deposit_cap = reserve_state.deposit_cap;
    
    assert_eq!(ltv, 9999);
    assert_eq!(liquidation_threshold, 10000);
    assert_eq!(liquidation_bonus, 2000);
    assert_eq!(borrow_cap, u64::MAX);
    assert_eq!(deposit_cap, u64::MAX);
}

#[test]
fn test_init_reserve_invalid_ltv_too_high() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();

    let (reserve_pubkey, _bump) = Pubkey::find_program_address(
        &[
            RESERVE_SEED.as_bytes(),
            ctx.market_pubkey.as_ref(),
            mint.as_ref(),
        ],
        &ctx.program_id,
    );

    let ix_data = InitReserveIxData {
        ltv: 10001, // 100.01% - invalid
        liquidation_threshold: 10000,
        liquidation_bonus: 500,
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(ctx.fee_payer.pubkey(), true),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect_err("LTV > 100% should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidInstructionData) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_INSTRUCTION_DATA as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_invalid_liquidation_threshold_too_high() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();

    let (reserve_pubkey, _bump) = Pubkey::find_program_address(
        &[
            RESERVE_SEED.as_bytes(),
            ctx.market_pubkey.as_ref(),
            mint.as_ref(),
        ],
        &ctx.program_id,
    );

    let ix_data = InitReserveIxData {
        ltv: 8000,
        liquidation_threshold: 10001, // 100.01% - invalid
        liquidation_bonus: 500,
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(ctx.fee_payer.pubkey(), true),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect_err("Liquidation threshold > 100% should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidInstructionData) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_INSTRUCTION_DATA as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_invalid_liquidation_bonus_too_high() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();

    let (reserve_pubkey, _bump) = Pubkey::find_program_address(
        &[
            RESERVE_SEED.as_bytes(),
            ctx.market_pubkey.as_ref(),
            mint.as_ref(),
        ],
        &ctx.program_id,
    );

    let ix_data = InitReserveIxData {
        ltv: 8000,
        liquidation_threshold: 8500,
        liquidation_bonus: 2001, // 20.01% - invalid (max is 20%)
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(ctx.fee_payer.pubkey(), true),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect_err("Liquidation bonus > 20% should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidInstructionData) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_INSTRUCTION_DATA as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_invalid_ltv_gte_liquidation_threshold() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();

    let (reserve_pubkey, _bump) = Pubkey::find_program_address(
        &[
            RESERVE_SEED.as_bytes(),
            ctx.market_pubkey.as_ref(),
            mint.as_ref(),
        ],
        &ctx.program_id,
    );

    let ix_data = InitReserveIxData {
        ltv: 8500,            // LTV >= liquidation_threshold is invalid
        liquidation_threshold: 8500,
        liquidation_bonus: 500,
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(ctx.fee_payer.pubkey(), true),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect_err("LTV >= liquidation threshold should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidInstructionData) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_INSTRUCTION_DATA as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_requires_market_owner() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    let unauthorized = Keypair::new();
    ctx.airdrop(&unauthorized.pubkey(), 1_000_000_000);

    let (reserve_pubkey, _bump) = Pubkey::find_program_address(
        &[
            RESERVE_SEED.as_bytes(),
            ctx.market_pubkey.as_ref(),
            mint.as_ref(),
        ],
        &ctx.program_id,
    );

    let ix_data = InitReserveIxData {
        ltv: 8000,
        liquidation_threshold: 8500,
        liquidation_bonus: 500,
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(unauthorized.pubkey(), true), // Wrong signer
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(
            vec![
                ctx.fee_payer.insecure_clone(),
                unauthorized,
            ],
            instruction,
        )
        .expect_err("Only lending market owner should be able to init reserve");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidAccountData) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_ACCOUNT_DATA as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_already_initialized() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    
    let (instruction1, _reserve_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        8000,
        8500,
        500,
        1_000_000_000,
        2_000_000_000,
    );

    // Initialize reserve first time
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction1)
        .expect("First initialization should succeed");

    // Try to initialize again with slightly different parameters
    let (instruction2, _) = ctx.build_init_reserve_instruction(
        &mint,
        7500, // Different LTV to make a different instruction
        8000,
        600,
        1_000_000_000,
        2_000_000_000,
    );
    
    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction2)
        .expect_err("Second initialization should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::AccountAlreadyInitialized) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::ACCOUNT_ALREADY_INITIALIZED as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_reserve_wrong_pda() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint();
    let wrong_reserve = Keypair::new();
    ctx.airdrop(&wrong_reserve.pubkey(), 1_000_000_000);

    let ix_data = InitReserveIxData {
        ltv: 8000,
        liquidation_threshold: 8500,
        liquidation_bonus: 500,
        borrow_cap: 1_000_000_000,
        deposit_cap: 2_000_000_000,
    };

    let data = encode_instruction(PlendInstructions::InitReserve, &ix_data);
    let instruction = Instruction {
        program_id: ctx.program_id,
        accounts: vec![
            AccountMeta::new(ctx.fee_payer.pubkey(), true),
            AccountMeta::new(wrong_reserve.pubkey(), false), // Wrong PDA
            AccountMeta::new_readonly(ctx.market_pubkey, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new_readonly(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
        ],
        data,
    };

    let err = ctx
        .send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction)
        .expect_err("Wrong PDA should fail");

    match err.err {
        TransactionError::InstructionError(_, InstructionError::InvalidSeeds) => {}
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, pinocchio::program_error::INVALID_SEEDS as u32)
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn test_init_multiple_reserves_same_market() {
    let mut ctx = initialize_lending_market();
    let mint1 = create_test_mint();
    let mint2 = create_test_mint();
    
    // Initialize first reserve
    let (instruction1, reserve_pubkey1) = ctx.build_init_reserve_instruction(
        &mint1,
        8000, 8500, 500, 1_000_000_000, 2_000_000_000,
    );

    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction1)
        .expect("First reserve initialization should succeed");

    // Initialize second reserve
    let (instruction2, reserve_pubkey2) = ctx.build_init_reserve_instruction(
        &mint2,
        7500, 8000, 600, 500_000_000, 1_500_000_000,
    );

    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], instruction2)
        .expect("Second reserve initialization should succeed");

    // Verify both reserves exist and have different addresses
    assert_ne!(reserve_pubkey1, reserve_pubkey2);
    
    let reserve1_state = ctx.reserve_state(&reserve_pubkey1);
    let reserve2_state = ctx.reserve_state(&reserve_pubkey2);
    
    // Copy packed fields to avoid alignment issues
    let mint1_bytes = reserve1_state.mint;
    let mint2_bytes = reserve2_state.mint;
    let ltv1 = reserve1_state.ltv;
    let ltv2 = reserve2_state.ltv;
    
    assert_eq!(mint1_bytes, mint1.to_bytes());
    assert_eq!(mint2_bytes, mint2.to_bytes());
    assert_eq!(ltv1, 8000);
    assert_eq!(ltv2, 7500);
}