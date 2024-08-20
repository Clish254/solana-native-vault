use crate::error::VaultError;
use crate::instruction::VaultInstruction;
use crate::state::{UserTransfers, Vault};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VaultInstruction::unpack(instruction_data)?;
    match instruction {
        VaultInstruction::Initialize {} => initialize(program_id, accounts),
        VaultInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        VaultInstruction::Withdraw {} => withdraw(program_id, accounts),
    }
}

pub fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let initializer = next_account_info(account_info_iter)?;
    let vault_pda = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (pda, bump_seed) =
        Pubkey::find_program_address(&[b"vault".as_ref(), initializer.key.as_ref()], program_id);
    if pda != *vault_pda.key {
        msg!("Invalid seeds for vault PDA");
        return Err(ProgramError::InvalidArgument);
    }
    let account_len = Vault::LEN;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            vault_pda.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            vault_pda.clone(),
            system_program.clone(),
        ],
        &[&[b"vault".as_ref(), initializer.key.as_ref(), &[bump_seed]]],
    )?;
    msg!("Vault PDA created: {}", pda);

    let mut account_data = Vault::try_from_slice(&vault_pda.data.borrow())?;

    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.discriminator = Vault::DISCRIMINATOR.to_string();
    account_data.deposited_amount = 0;
    account_data.withdrawn_amount = 0;
    account_data.initialized = true;
    account_data.owner = *initializer.key;

    account_data.serialize(&mut &mut vault_pda.data.borrow_mut()[..])?;
    Ok(())
}

pub fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let depositor = next_account_info(account_info_iter)?;
    let vault_pda = next_account_info(account_info_iter)?;
    let user_transfers_pda = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    if !depositor.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (vpda, vbump_seed) =
        Pubkey::find_program_address(&[b"vault".as_ref(), depositor.key.as_ref()], program_id);
    if vpda != *vault_pda.key {
        msg!("Invalid seeds for vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut vault_account_data = Vault::try_from_slice(&vault_pda.data.borrow())?;

    if !vault_account_data.is_initialized() {
        msg!("Vault account has not been initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    let (transfers_pda, transfers_pda_bump_seed) = Pubkey::find_program_address(
        &[b"user_transfers".as_ref(), vault_pda.key.as_ref()],
        program_id,
    );
    if transfers_pda != *user_transfers_pda.key {
        msg!("Invalid seeds for user transfers PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let account_len = UserTransfers::LEN;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);
    let mut transfers_account_data =
        UserTransfers::try_from_slice(&user_transfers_pda.data.borrow())?;
    if !transfers_account_data.is_initialized() {
        invoke_signed(
            &system_instruction::create_account(
                depositor.key,
                user_transfers_pda.key,
                rent_lamports,
                account_len.try_into().unwrap(),
                program_id,
            ),
            &[
                depositor.clone(),
                user_transfers_pda.clone(),
                system_program.clone(),
            ],
            &[&[
                b"user_transfers".as_ref(),
                depositor.key.as_ref(),
                &[transfers_pda_bump_seed],
            ]],
        )?;
        msg!("User transfers PDA created: {}", transfers_pda);
    }

    let transfer_instruction = system_instruction::transfer(depositor.key, vault_pda.key, amount);
    invoke(
        &transfer_instruction,
        &[
            depositor.clone(), // Signer's account
            vault_pda.clone(), // PDA's account
        ],
    )?;
    msg!("{} lamports transferred to user_transfers PDA", amount);

    vault_account_data.deposited_amount = vault_account_data
        .deposited_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault_account_data.serialize(&mut &mut vault_pda.data.borrow_mut()[..])?;

    if transfers_account_data.is_initialized() {
        transfers_account_data.deposited_amount = transfers_account_data
            .deposited_amount
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    } else {
        transfers_account_data.deposited_amount = transfers_account_data
            .deposited_amount
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        transfers_account_data.withdrawn_amount = 0;
        transfers_account_data.initialized = true;
        transfers_account_data.owner = *depositor.key;
        transfers_account_data.vault = *vault_pda.key;
    }
    transfers_account_data.serialize(&mut &mut user_transfers_pda.data.borrow_mut()[..])?;
    Ok(())
}

pub fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let withdrawer = next_account_info(account_info_iter)?;
    let vault_pda = next_account_info(account_info_iter)?;
    let user_transfers_pda = next_account_info(account_info_iter)?;

    if !withdrawer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (vpda, vbump_seed) =
        Pubkey::find_program_address(&[b"vault".as_ref(), withdrawer.key.as_ref()], program_id);
    if vpda != *vault_pda.key {
        msg!("Invalid seeds for vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut vault_account_data = Vault::try_from_slice(&vault_pda.data.borrow())?;

    if !vault_account_data.is_initialized() {
        msg!("Vault account has not been initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    let (transfers_pda, transfers_pda_bump_seed) = Pubkey::find_program_address(
        &[b"user_transfers".as_ref(), vault_pda.key.as_ref()],
        program_id,
    );
    if transfers_pda != *user_transfers_pda.key {
        msg!("Invalid seeds for user transfers PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mut transfers_account_data =
        UserTransfers::try_from_slice(&user_transfers_pda.data.borrow())?;
    if !transfers_account_data.is_initialized() {
        msg!("User transfers account has not been initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    let user_available_amount = transfers_account_data
        .deposited_amount
        .checked_sub(transfers_account_data.withdrawn_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let withdraw_amount = user_available_amount * 10 / 100;
    let available_amount = transfers_account_data
        .deposited_amount
        .checked_sub(transfers_account_data.withdrawn_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if available_amount < withdraw_amount {
        return Err(VaultError::InvalidWithdrawAmount.into());
    }

    **vault_pda.try_borrow_mut_lamports()? -= withdraw_amount;
    **withdrawer.try_borrow_mut_lamports()? += withdraw_amount;
    msg!("{} lamports transferred to user", withdraw_amount);

    transfers_account_data.withdrawn_amount = transfers_account_data
        .withdrawn_amount
        .checked_add(withdraw_amount)
        .unwrap();
    transfers_account_data.serialize(&mut &mut user_transfers_pda.data.borrow_mut()[..])?;

    vault_account_data.withdrawn_amount = vault_account_data
        .withdrawn_amount
        .checked_add(withdraw_amount)
        .unwrap();
    vault_account_data.serialize(&mut &mut vault_pda.data.borrow_mut()[..])?;
    Ok(())
}
