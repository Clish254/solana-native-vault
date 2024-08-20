use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub discriminator: String,
    pub deposited_amount: u64,
    pub withdrawn_amount: u64,
    pub initialized: bool,
    pub owner: Pubkey,
}

impl Vault {
    pub const DISCRIMINATOR: &'static str = "vault";

    pub const LEN: usize = {
        let discriminator = 4 + Vault::DISCRIMINATOR.len();
        let amounts = 2 * 8;
        let initialized = 1;
        let owner = 32;
        discriminator + amounts + initialized + owner
    };
}

impl Sealed for Vault {}

impl IsInitialized for Vault {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserTransfers {
    pub discriminator: String,
    pub deposited_amount: u64,
    pub withdrawn_amount: u64,
    pub initialized: bool,
    pub owner: Pubkey,
    pub vault: Pubkey,
}

impl UserTransfers {
    pub const DISCRIMINATOR: &'static str = "transfers";
    pub const LEN: usize = {
        let discriminator = 4 + UserTransfers::DISCRIMINATOR.len();
        let amounts = 2 * 8;
        let initialized = 1;
        let pubkeys = 2 * 32;
        discriminator + amounts + initialized + pubkeys
    };
}

impl Sealed for UserTransfers {}

impl IsInitialized for UserTransfers {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}
