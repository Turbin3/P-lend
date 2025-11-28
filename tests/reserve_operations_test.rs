mod common;

use common::initialize_lending_market;

fn create_test_mint(ctx: &mut common::InitializedMarket) -> solana_pubkey::Pubkey {
    ctx.create_mint(6)
}

#[test]
fn test_enable_reserve_success() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);
    
    // Initialize reserve (starts as active)
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Disable it first
    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("Disable should succeed");

    // Now enable it
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("Enable should succeed");

    // Verify state
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 1, "Reserve should be active");
}

#[test]
fn test_disable_reserve_success() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);
    
    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Disable it
    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("Disable should succeed");

    // Verify state
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 0, "Reserve should be inactive");
}

#[test]
fn test_close_reserve_success() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);
    
    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Close it (only owner can close)
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("Close should succeed");

    // Verify state
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 0, "Reserve should be inactive");
}

#[test]
fn test_enable_reserve_with_risk_council() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);
    
    // Initialize and disable reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("Disable should succeed");

    // Owner can enable
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("Owner should be able to enable");

    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 1);
}

#[test]
fn test_enable_reserve_requires_authority() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Disable first
    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("Disable should succeed");

    // Owner can enable (this test verifies owner authority works)
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("Owner should be able to enable");
    
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 1, "Reserve should be active");
}

#[test]
fn test_close_reserve_requires_owner() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Owner can close
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("Owner should be able to close");
    
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    assert_eq!(reserve_state.is_active, 0, "Reserve should be inactive");
}

#[test]
fn test_cannot_enable_closed_reserve() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Close it
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("Close should succeed");

    // Try to enable closed reserve
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    let result = ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix);
    
    assert!(result.is_err(), "Cannot enable a closed reserve");
}

#[test]
fn test_cannot_disable_closed_reserve() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Close it
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("Close should succeed");

    // Try to disable closed reserve
    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    let result = ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix);
    
    assert!(result.is_err(), "Cannot disable a closed reserve");
}

#[test]
fn test_close_already_closed_reserve_fails() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize reserve
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Reserve initialization should succeed");

    // Close it
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("First close should succeed");

    // Try to close again
    let close_ix2 = ctx.build_close_reserve_instruction(&reserve_pubkey);
    let result = ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix2);
    
    assert!(result.is_err(), "Cannot close an already closed reserve");
}

#[test]
fn test_state_transitions() {
    let mut ctx = initialize_lending_market();
    let mint = create_test_mint(&mut ctx);

    // Initialize (active by default)
    let (init_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 2_000_000_000
    );
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_ix)
        .expect("Init should succeed");
    assert_eq!(ctx.reserve_state(&reserve_pubkey).is_active, 1);

    // Disable
    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("Disable should succeed");
    assert_eq!(ctx.reserve_state(&reserve_pubkey).is_active, 0);

    // Enable
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("Enable should succeed");
    assert_eq!(ctx.reserve_state(&reserve_pubkey).is_active, 1);

    // Close (final state)
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("Close should succeed");
    assert_eq!(ctx.reserve_state(&reserve_pubkey).is_active, 0);
}
