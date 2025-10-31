use core::{mem, ptr};
use std::str::FromStr;

use p_lend::{
    instructions::init_lending_market::{process_init_lending_market, InitLendingMarketIxData},
    state::LendingMarketState,
    DataLen, LendingMarketInstruction, StateDefinition, ID,
};
use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    sysvars::rent::{Rent as PinRent, RENT_ID},
};
use solana_sdk::{pubkey::Pubkey as SolPubkey, signature::Keypair, signer::Signer};

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

struct TestAccount {
    info: AccountInfo,
    _backing: Vec<u64>,
}

impl TestAccount {
    fn new(
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

    fn info(&self) -> AccountInfo {
        self.info
    }
}

fn serialize_struct<T>(value: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((value as *const T) as *const u8, mem::size_of::<T>()) }
}

#[test]
fn test_init_lending_market() {
    let program_id = ID;
    let owner_keypair = Keypair::new();
    let owner_pubkey = owner_keypair.pubkey().to_bytes();
    let quote_currency = [42u8; 32];
    let risk_council = Keypair::new().pubkey().to_bytes();

    let (lending_market_solana, _bump) = SolPubkey::find_program_address(
        &[LendingMarketState::SEED.as_bytes(), owner_pubkey.as_slice()],
        &SolPubkey::new_from_array(program_id),
    );
    let lending_market_pubkey = lending_market_solana.to_bytes();

    #[allow(deprecated)]
    let rent = PinRent {
        lamports_per_byte_year: 3_480,
        exemption_threshold: 2.0,
        burn_percent: 50,
    };
    let rent_bytes = serialize_struct(&rent);
    let system_program = SolPubkey::from_str("11111111111111111111111111111111")
        .unwrap()
        .to_bytes();

    let payer = TestAccount::new(owner_pubkey, system_program, 1_000_000_000, 0, true, true);
    let market = TestAccount::new(lending_market_pubkey, system_program, 0, 0, false, true);
    let rent_account = {
        let account = TestAccount::new(RENT_ID, RENT_ID, 0, rent_bytes.len(), false, false);
        {
            let info = account.info();
            let mut data = info.try_borrow_mut_data().unwrap();
            data.copy_from_slice(rent_bytes);
        }
        account
    };

    let accounts = [payer.info(), market.info(), rent_account.info()];
    let state_len = <LendingMarketState as DataLen>::LEN;

    let ix_data = InitLendingMarketIxData {
        lending_market_owner: owner_pubkey,
        quote_currency,
        risk_council,
    };

    let mut data = Vec::with_capacity(1 + InitLendingMarketIxData::LEN);
    data.push(LendingMarketInstruction::InitLendingMarket as u8);
    data.extend_from_slice(serialize_struct(&ix_data));

    process_init_lending_market(&program_id, &accounts, &data[1..]).expect("init should succeed");

    assert_eq!(market.info().owner(), &program_id);
    assert_eq!(market.info().data_len(), state_len);
    let required = rent.minimum_balance(state_len);
    assert_eq!(market.info().lamports(), required);
    assert_eq!(payer.info().lamports(), 1_000_000_000 - required);
}
