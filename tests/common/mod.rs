use bytemuck::try_from_bytes;
use litesvm::{
    types::{FailedTransactionMetadata, TransactionMetadata},
    LiteSVM,
};
use litesvm_token::{spl_token, CreateMint, CreateAssociatedTokenAccount, MintTo};
use pinocchio::sysvars::rent::RENT_ID;
use pinocchio_system::ID as SYSTEM_PROGRAM_ID;
use plend::{
    constants::{RESERVE_SEED, RESERVE_VAULT_SEED},
    helper::utils::DataLen,
    instructions::{
        init_lending_market::InitLendingMarketIxData, 
        init_reserve::InitReserveIxData,
        set_emergency_mode::SetEmergencyModeIxData,
        update_lending_market_owner::UpdateLendingMarketOwnerIxData,
        update_risk_council::UpdateRiskCouncilIxData, 
        enable_reserve::EnableReserveIxData,
        disable_reserve::DisableReserveIxData,
        close_reserve::CloseReserveIxData,
        supply_liquidity::SupplyLiquidityIxData,
        PlendInstructions,
    },
    state::{LendingMarketState, ReserveState},
    ID,
};
use solana_instruction::{account_meta::AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::{v0, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_sdk_ids::system_program;
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

const PROGRAM_ID: Pubkey = Pubkey::new_from_array(ID);
const MARKET_SEED: &[u8] = b"lending_market";
const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array(spl_token::ID.to_bytes());

pub fn serialize_struct<T>(value: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts((value as *const T) as *const u8, core::mem::size_of::<T>())
    }
}

pub fn setup_svm_and_program() -> (LiteSVM, Keypair, Pubkey) {
    let mut svm = LiteSVM::new();
    let fee_payer = Keypair::new();
    svm.airdrop(&fee_payer.pubkey(), 1_000_000_000)
        .expect("failed to airdrop to fee payer");
    svm.add_program_from_file(PROGRAM_ID, "./target/deploy/plend.so")
        .expect("failed to load program binary");
    (svm, fee_payer, PROGRAM_ID)
}

pub fn build_and_send_transaction(
    svm: &mut LiteSVM,
    signers: &[&Keypair],
    instructions: Vec<Instruction>,
) -> Result<TransactionMetadata, FailedTransactionMetadata> {
    assert!(
        !signers.is_empty(),
        "transactions require at least one signer (payer)"
    );
    let payer = signers[0].pubkey();
    let message = v0::Message::try_compile(&payer, &instructions, &[], svm.latest_blockhash())
        .expect("failed to compile transaction message");
    let tx = VersionedTransaction::try_new(VersionedMessage::V0(message), signers)
        .expect("bad signer set");
    svm.send_transaction(tx)
}

pub struct InitializedMarket {
    pub program_id: Pubkey,
    pub market_pubkey: Pubkey,
    pub fee_payer: Keypair,
    pub risk_council: Keypair,
    svm: LiteSVM,
}

impl InitializedMarket {
    pub fn market_state(&self) -> LendingMarketState {
        let account = self
            .svm
            .get_account(&self.market_pubkey)
            .expect("lending market account missing");
        if account.data.len() < LendingMarketState::LEN {
            panic!(
                "market account too small: {} < {}",
                account.data.len(),
                LendingMarketState::LEN
            );
        }
        let data = &account.data[..LendingMarketState::LEN];
        *try_from_bytes::<LendingMarketState>(data).expect("invalid lending market account state")
    }

    pub fn reserve_state(&self, reserve_pubkey: &Pubkey) -> ReserveState {
        let account = self
            .svm
            .get_account(reserve_pubkey)
            .expect("reserve account missing");
        if account.data.len() < ReserveState::LEN {
            panic!(
                "reserve account too small: {} < {}",
                account.data.len(),
                ReserveState::LEN
            );
        }
        let data = &account.data[..ReserveState::LEN];
        *try_from_bytes::<ReserveState>(data).expect("invalid reserve account state")
    }

    pub fn owner_pubkey(&self) -> [u8; 32] {
        self.fee_payer.pubkey().to_bytes()
    }

    pub fn risk_council_pubkey(&self) -> [u8; 32] {
        self.risk_council.pubkey().to_bytes()
    }

    pub fn send_instruction(
        &mut self,
        signers: Vec<Keypair>,
        instruction: Instruction,
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        let signer_refs: Vec<&Keypair> = signers.iter().collect();
        build_and_send_transaction(&mut self.svm, &signer_refs, vec![instruction])
    }

    pub fn airdrop(&mut self, recipient: &Pubkey, lamports: u64) {
        self.svm
            .airdrop(recipient, lamports)
            .expect("airdrop failed unexpectedly");
    }

    pub fn create_mint(&mut self, decimals: u8) -> Pubkey {
        CreateMint::new(&mut self.svm, &self.fee_payer)
            .decimals(decimals)
            .send()
            .expect("failed to create mint")
    }

    pub fn get_vault(&self, vault_pubkey: &Pubkey) -> Option<solana_account::Account> {
        self.svm.get_account(vault_pubkey)
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<solana_account::Account> {
        self.svm.get_account(pubkey)
    }

    pub fn verify_vault_created(&self, vault_pubkey: &Pubkey, expected_mint: &Pubkey) -> bool {
        if let Some(account) = self.svm.get_account(vault_pubkey) {
            account.lamports > 0
        } else {
            false
        }
    }

    pub fn create_token_account(&mut self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        CreateAssociatedTokenAccount::new(&mut self.svm, &self.fee_payer, mint)
            .owner(owner)
            .send()
            .expect("failed to create associated token account")
    }

    pub fn mint_tokens(&mut self, mint: &Pubkey, destination: &Pubkey, amount: u64) {
        MintTo::new(&mut self.svm, &self.fee_payer, mint, destination, amount)
            .send()
            .expect("failed to mint tokens")
    }

    pub fn build_init_reserve_instruction(
        &self,
        mint: &Pubkey,
        ltv: u16,
        liquidation_threshold: u16,
        liquidation_bonus: u16,
        borrow_cap: u64,
        deposit_cap: u64,
    ) -> (Instruction, Pubkey, Pubkey) {
        let (reserve_pubkey, _bump) = Pubkey::find_program_address(
            &[
                RESERVE_SEED.as_bytes(),
                self.market_pubkey.as_ref(),
                mint.as_ref(),
            ],
            &self.program_id,
        );

        let (vault_pubkey, _vault_bump) = Pubkey::find_program_address(
            &[
                RESERVE_VAULT_SEED.as_bytes(),
                reserve_pubkey.as_ref(),
            ],
            &self.program_id,
        );

        let ix_data = InitReserveIxData {
            ltv,
            liquidation_threshold,
            liquidation_bonus,
            borrow_cap,
            deposit_cap,
        };

        let mut data = Vec::with_capacity(1 + InitReserveIxData::LEN);
        data.push(PlendInstructions::InitReserve as u8);
        data.extend_from_slice(serialize_struct(&ix_data));

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(reserve_pubkey, false),
                AccountMeta::new_readonly(self.market_pubkey, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(vault_pubkey, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
                AccountMeta::new(Pubkey::new_from_array(SYSTEM_PROGRAM_ID), false),
            ],
            data,
        };

        (instruction, reserve_pubkey, vault_pubkey)
    }

    pub fn build_enable_reserve_instruction(&self, reserve_pubkey: &Pubkey) -> Instruction {
        let ix_data = EnableReserveIxData {};
        let mut data = Vec::with_capacity(1 + EnableReserveIxData::LEN);
        data.push(PlendInstructions::EnableReserve as u8);
        data.extend_from_slice(serialize_struct(&ix_data));

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(*reserve_pubkey, false),
                AccountMeta::new_readonly(self.market_pubkey, false),
            ],
            data,
        }
    }

    pub fn build_disable_reserve_instruction(&self, reserve_pubkey: &Pubkey) -> Instruction {
        let ix_data = DisableReserveIxData {};
        let mut data = Vec::with_capacity(1 + DisableReserveIxData::LEN);
        data.push(PlendInstructions::DisableReserve as u8);
        data.extend_from_slice(serialize_struct(&ix_data));

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(*reserve_pubkey, false),
                AccountMeta::new_readonly(self.market_pubkey, false),
            ],
            data,
        }
    }

    pub fn build_close_reserve_instruction(&self, reserve_pubkey: &Pubkey) -> Instruction {
        let ix_data = CloseReserveIxData {};
        let mut data = Vec::with_capacity(1 + CloseReserveIxData::LEN);
        data.push(PlendInstructions::CloseReserve as u8);
        data.extend_from_slice(serialize_struct(&ix_data));

        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(*reserve_pubkey, false),
                AccountMeta::new_readonly(self.market_pubkey, false),
            ],
            data,
        }
    }

    pub fn build_set_emergency_mode_instruction(&self, enable: u8) -> Instruction {
        let ix_data = SetEmergencyModeIxData { enable };
        let mut data = Vec::with_capacity(1 + SetEmergencyModeIxData::LEN);
        data.push(PlendInstructions::SetEmergencyMode as u8);
        data.extend_from_slice(serialize_struct(&ix_data));
        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.risk_council.pubkey(), true),
                AccountMeta::new(self.market_pubkey, false),
            ],
            data,
        }
    }

    pub fn build_update_risk_council_instruction(&self, new_risk: [u8; 32]) -> Instruction {
        let ix_data = UpdateRiskCouncilIxData {
            new_risk_council: new_risk,
        };
        let mut data = Vec::with_capacity(1 + UpdateRiskCouncilIxData::LEN);
        data.push(PlendInstructions::UpdateRiskCouncil as u8);
        data.extend_from_slice(serialize_struct(&ix_data));
        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(self.market_pubkey, false),
            ],
            data,
        }
    }

    pub fn build_update_owner_instruction(&self, new_owner: [u8; 32]) -> Instruction {
        let ix_data = UpdateLendingMarketOwnerIxData { new_owner };
        let mut data = Vec::with_capacity(1 + UpdateLendingMarketOwnerIxData::LEN);
        data.push(PlendInstructions::UpdateLendingMarketOwner as u8);
        data.extend_from_slice(serialize_struct(&ix_data));
        Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.fee_payer.pubkey(), true),
                AccountMeta::new(self.market_pubkey, false),
            ],
            data,
        }
    }
}

pub fn initialize_lending_market() -> InitializedMarket {
    let (mut svm, fee_payer, program_id) = setup_svm_and_program();
    let risk_council = Keypair::new();
    svm.airdrop(&risk_council.pubkey(), 100_000_000)
        .expect("failed to fund risk council");

    let (market_pubkey, _bump) =
        Pubkey::find_program_address(&[MARKET_SEED, fee_payer.pubkey().as_ref()], &program_id);

    let ix_data = InitLendingMarketIxData {
        lending_market_owner: fee_payer.pubkey().to_bytes(),
        quote_currency: [42u8; 32],
        risk_council: risk_council.pubkey().to_bytes(),
    };

    let mut data = Vec::with_capacity(1 + InitLendingMarketIxData::LEN);
    data.push(PlendInstructions::InitLendingMarket as u8);
    data.extend_from_slice(serialize_struct(&ix_data));

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(fee_payer.pubkey(), true),
            AccountMeta::new(market_pubkey, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(RENT_ID), false),
            AccountMeta::new(system_program::ID, false),
        ],
        data,
    };

    build_and_send_transaction(&mut svm, &[&fee_payer], vec![instruction])
        .expect("initialization transaction failed");

    InitializedMarket {
        program_id,
        market_pubkey,
        fee_payer,
        risk_council,
        svm,
    }
}
