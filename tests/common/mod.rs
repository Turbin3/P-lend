use core::{mem, ptr};
use std::str::FromStr;

use p_lend::{
    helper::utils::try_from_account_info_mut,
    instructions::init_lending_market::{process_init_lending_market, InitLendingMarketIxData},
    state::LendingMarketState,
    StateDefinition, ID,
};
use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    sysvars::rent::{Rent as PinRent, RENT_ID},
};
use solana_sdk::pubkey::Pubkey as SolPubkey;

#[repr(C)]
#[derive(Clone, Copy)]
struct AccountLayout {
    borrow_state: u8,
    is_signer: u8,
    is_writable: u8,
    executable: u8,
    resize_delta: i32,
    key: Pubkey,
    owner: Pubkey,
    lamports: u64,
    data_len: u64,
}

#[derive(Clone)]
pub struct TestAccount {
    info: AccountInfo,
    _backing: Vec<u64>,
}

impl TestAccount {
    pub fn new(
        key: Pubkey,
        owner: Pubkey,
        lamports: u64,
        data_len: usize,
        is_signer: bool,
        is_writable: bool,
    ) -> Self {
        let header = mem::size_of::<AccountLayout>();
        let total_bytes = header + data_len;
        let words = (total_bytes + 7) / 8;
        let mut backing = vec![0u64; words];
        let header_ptr = backing.as_mut_ptr() as *mut AccountLayout;

        unsafe {
            ptr::write(
                header_ptr,
                AccountLayout {
                    borrow_state: u8::MAX,
                    is_signer: is_signer as u8,
                    is_writable: is_writable as u8,
                    executable: 0,
                    resize_delta: 0,
                    key,
                    owner,
                    lamports,
                    data_len: data_len as u64,
                },
            );
        }

        let info = unsafe { mem::transmute::<*mut AccountLayout, AccountInfo>(header_ptr) };
        Self {
            info,
            _backing: backing,
        }
    }

    pub fn info(&self) -> AccountInfo {
        self.info
    }
}

pub fn serialize_struct<T>(value: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((value as *const T) as *const u8, mem::size_of::<T>()) }
}

pub struct InitializedMarket {
    pub program_id: Pubkey,
    owner_pubkey: Pubkey,
    owner_seed: Pubkey,
    pub risk_council_pubkey: Pubkey,
    owner_account: TestAccount,
    _rent_account: TestAccount,
    pub market: TestAccount,
    pub risk_council_account: TestAccount,
}

impl InitializedMarket {
    pub fn market_state(&self) -> LendingMarketState {
        unsafe {
            let info = self.market.info();
            *try_from_account_info_mut::<LendingMarketState>(&info).unwrap()
        }
    }

    pub fn owner_account_info(&self) -> AccountInfo {
        self.owner_account.info()
    }

    pub fn owner_pubkey(&self) -> &Pubkey {
        &self.owner_pubkey
    }

    pub fn owner_seed(&self) -> &Pubkey {
        &self.owner_seed
    }
}

pub fn initialize_lending_market() -> InitializedMarket {
    let program_id = ID;
    let system_program = system_program();

    let owner_seed = [1u8; 32];
    let risk_seed = [2u8; 32];

    let (market_pubkey, _bump) = SolPubkey::find_program_address(
        &[LendingMarketState::SEED.as_bytes(), owner_seed.as_slice()],
        &SolPubkey::new_from_array(program_id),
    );
    let market_pubkey = market_pubkey.to_bytes();

    #[allow(deprecated)]
    let rent = PinRent {
        lamports_per_byte_year: 3_480,
        exemption_threshold: 2.0,
        burn_percent: 50,
    };
    let rent_bytes = serialize_struct(&rent).to_vec();

    let payer_account = TestAccount::new(owner_seed, system_program, 1_000_000_000, 0, true, true);
    let market = TestAccount::new(market_pubkey, system_program, 0, 0, false, true);
    let rent_account = TestAccount::new(RENT_ID, RENT_ID, 0, rent_bytes.len(), false, false);
    {
        let info = rent_account.info();
        let mut data = info.try_borrow_mut_data().unwrap();
        data.copy_from_slice(&rent_bytes);
    }

    let ix_data = InitLendingMarketIxData {
        lending_market_owner: owner_seed,
        quote_currency: [42u8; 32],
        risk_council: risk_seed,
    };

    let ix_bytes = serialize_struct(&ix_data).to_vec();
    let accounts = [payer_account.info(), market.info(), rent_account.info()];

    process_init_lending_market(&program_id, &accounts, &ix_bytes).unwrap();

    let market_state = unsafe {
        let info = market.info();
        *try_from_account_info_mut::<LendingMarketState>(&info).unwrap()
    };

    let owner_account = TestAccount::new(
        market_state.lending_market_owner,
        system_program,
        1_000_000_000,
        0,
        true,
        true,
    );
    let risk_council_account =
        TestAccount::new(market_state.risk_council, system_program, 0, 0, true, false);

    InitializedMarket {
        program_id,
        owner_pubkey: market_state.lending_market_owner,
        owner_seed,
        risk_council_pubkey: market_state.risk_council,
        owner_account,
        market,
        _rent_account: rent_account,
        risk_council_account,
    }
}

pub fn system_program() -> Pubkey {
    SolPubkey::from_str("11111111111111111111111111111111")
        .unwrap()
        .to_bytes()
}
