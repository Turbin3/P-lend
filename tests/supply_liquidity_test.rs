mod common;

use common::{initialize_lending_market, serialize_struct};
use plend::{
    helper::utils::DataLen,
    instructions::{supply_liquidity::SupplyLiquidityIxData, PlendInstructions},
};
use solana_instruction::{account_meta::AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(litesvm_token::spl_token::ID.to_bytes());

/// Helper to build supply_liquidity instruction
fn build_supply_liquidity_instruction(
    program_id: &Pubkey,
    user: &Pubkey,
    user_token_account: &Pubkey,
    reserve: &Pubkey,
    reserve_vault: &Pubkey,
    amount: u64,
) -> Instruction {
    let ix_data = SupplyLiquidityIxData { amount };
    let mut data = Vec::with_capacity(1 + SupplyLiquidityIxData::LEN);
    data.push(PlendInstructions::SupplyLiquidity as u8);
    data.extend_from_slice(serialize_struct(&ix_data));

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_token_account, false),
            AccountMeta::new(*reserve, false),
            AccountMeta::new(*reserve_vault, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data,
    }
}

#[test]
fn test_supply_liquidity_success() {
    let mut ctx = initialize_lending_market();
    
    // Create mint and reserve
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        8000,  // 80% LTV
        8500,  // 85% liquidation threshold
        500,   // 5% liquidation bonus
        1_000_000_000,  // 1000 tokens borrow cap
        10_000_000_000, // 10000 tokens deposit cap
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    // Enable reserve
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    // Create user and fund their token account
    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Supply liquidity
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        500_000, // 500k tokens
    );

    ctx.send_instruction(vec![user], supply_ix)
        .expect("supply liquidity failed");

    // Verify reserve state updated
    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    let available_liquidity = reserve_state.available_liquidity;
    let total_supply = reserve_state.total_supply;
    assert_eq!(available_liquidity, 500_000);
    assert_eq!(total_supply, 500_000);
}

#[test]
fn test_supply_liquidity_multiple_deposits() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 2_000_000);

    // First deposit
    let supply_ix1 = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        300_000,
    );

    ctx.send_instruction(vec![user.insecure_clone()], supply_ix1)
        .expect("first supply failed");

    let reserve_state_1 = ctx.reserve_state(&reserve_pubkey);
    let available_liquidity_1 = reserve_state_1.available_liquidity;
    let total_supply_1 = reserve_state_1.total_supply;
    assert_eq!(available_liquidity_1, 300_000);
    assert_eq!(total_supply_1, 300_000);

    // Second deposit
    let supply_ix2 = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        700_000,
    );

    ctx.send_instruction(vec![user.insecure_clone()], supply_ix2)
        .expect("second supply failed");

    // Verify cumulative state
    let reserve_state_2 = ctx.reserve_state(&reserve_pubkey);
    let available_liquidity_2 = reserve_state_2.available_liquidity;
    let total_supply_2 = reserve_state_2.total_supply;
    assert_eq!(available_liquidity_2, 1_000_000);
    assert_eq!(total_supply_2, 1_000_000);
}

#[test]
fn test_supply_liquidity_zero_amount() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Try to supply zero amount
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        0, // Zero amount
    );

    let result = ctx.send_instruction(vec![user], supply_ix);
    assert!(result.is_err(), "should reject zero amount");
}

#[test]
fn test_supply_liquidity_inactive_reserve() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    // Enable then disable the reserve to make it inactive
    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let disable_ix = ctx.build_disable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], disable_ix)
        .expect("disable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Try to supply to inactive reserve
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        500_000,
    );

    let result = ctx.send_instruction(vec![user], supply_ix);
    assert!(result.is_err(), "should reject deposit to inactive reserve");
}

#[test]
fn test_supply_liquidity_closed_reserve() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    // Close the reserve
    let close_ix = ctx.build_close_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], close_ix)
        .expect("close reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Try to supply to closed reserve
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        500_000,
    );

    let result = ctx.send_instruction(vec![user], supply_ix);
    assert!(result.is_err(), "should reject deposit to closed reserve");
}

#[test]
fn test_supply_liquidity_exceeds_deposit_cap() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        8000,
        8500,
        500,
        1_000_000_000,
        1_000_000, // Low deposit cap: 1M tokens
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 5_000_000);

    // Try to deposit more than cap
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        2_000_000, // Exceeds 1M cap
    );

    let result = ctx.send_instruction(vec![user], supply_ix);
    assert!(result.is_err(), "should reject deposit exceeding cap");
}

#[test]
fn test_supply_liquidity_wrong_vault() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, _vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Use a fake vault address instead of the real one
    let fake_vault = Pubkey::new_unique();

    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &fake_vault, // Wrong vault!
        500_000,
    );

    let result = ctx.send_instruction(vec![user], supply_ix);
    assert!(result.is_err(), "should reject wrong vault address");
}

#[test]
#[should_panic(expected = "bad signer set")]
fn test_supply_liquidity_requires_user_signature() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint, 8000, 8500, 500, 1_000_000_000, 10_000_000_000,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 1_000_000);

    // Build instruction that requires user signature
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        500_000,
    );

    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], supply_ix)
        .expect("should panic before reaching this");
}

#[test]
fn test_supply_liquidity_respects_deposit_cap_edge_case() {
    let mut ctx = initialize_lending_market();
    
    let mint = ctx.create_mint(6);
    let deposit_cap = 1_000_000;
    let (init_reserve_ix, reserve_pubkey, vault_pubkey) = ctx.build_init_reserve_instruction(
        &mint,
        8000,
        8500,
        500,
        1_000_000_000,
        deposit_cap,
    );
    
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], init_reserve_ix)
        .expect("init reserve failed");

    let enable_ix = ctx.build_enable_reserve_instruction(&reserve_pubkey);
    ctx.send_instruction(vec![ctx.fee_payer.insecure_clone()], enable_ix)
        .expect("enable reserve failed");

    let user = Keypair::new();
    ctx.airdrop(&user.pubkey(), 100_000_000);
    
    let user_token_account = ctx.create_token_account(&user.pubkey(), &mint);
    ctx.mint_tokens(&mint, &user_token_account, 2_000_000);

    // Deposit exactly at cap should succeed
    let supply_ix = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        deposit_cap,
    );

    ctx.send_instruction(vec![user.insecure_clone()], supply_ix)
        .expect("deposit at cap should succeed");

    let reserve_state = ctx.reserve_state(&reserve_pubkey);
    let total_supply = reserve_state.total_supply;
    assert_eq!(total_supply, deposit_cap);

    // Now any additional deposit should fail
    let supply_ix2 = build_supply_liquidity_instruction(
        &ctx.program_id,
        &user.pubkey(),
        &user_token_account,
        &reserve_pubkey,
        &vault_pubkey,
        1, // Even 1 token over cap
    );

    let result = ctx.send_instruction(vec![user], supply_ix2);
    assert!(result.is_err(), "should reject deposit over cap");
}
