# Vault Program

This Solana program provides a basic vault functionality that allows users to initialize a vault, deposit funds into it, and withdraw a portion of their deposited funds. The program manages vaults through Program Derived Addresses (PDAs) and handles deposits and withdrawals securely, ensuring that only authorized users can perform these actions.

## Features

- **Initialize Vault**: Users can create a new vault account tied to their public key. This vault is used to manage and track deposits and withdrawals.
- **Deposit Funds**: Users can deposit funds (in lamports) into their vault. The program tracks the total amount deposited.
- **Withdraw Funds**: Users can withdraw a portion (10%) of the funds they've deposited into their vault. The program ensures that users cannot withdraw more than what is available.

## Program Structure

### Instructions

- **Initialize**: Creates a new vault PDA for the user and initializes it with a zero balance.
- **Deposit**: Transfers lamports from the user's account to their vault and updates the deposit record.
- **Withdraw**: Allows the user to withdraw 10% of their available deposit from the vault.

### Account Structures

- **Vault**: Stores information about the vault, including the total deposited amount, withdrawn amount, and the owner of the vault.
- **UserTransfers**: Tracks the deposited and withdrawn amounts for a specific user within a vault.

### Error Handling

The program uses `ProgramError` for standard error handling and a custom `VaultError` for specific vault-related errors such as invalid withdraw amounts.

## How to Use

1. **Initialize a Vault**:

   - Call the `Initialize` instruction to create and initialize a vault PDA. This requires the user's public key and the program ID.

2. **Deposit Funds**:

   - Call the `Deposit` instruction with the amount to deposit. This transfers the specified amount from the user's account to the vault PDA.

3. **Withdraw Funds**:

   - Call the `Withdraw` instruction to withdraw 10% of the deposited amount. The program checks the available balance and ensures the withdrawal is valid.

## Program Flow

1. **Initialization**:

   - The user sends an `Initialize` instruction.
   - A vault PDA is created using the user's public key.
   - The vault is initialized with zero deposited and withdrawn amounts.

2. **Deposit**:

   - The user sends a `Deposit` instruction with the amount to deposit.
   - The program transfers the specified amount from the user's account to the vault.
   - The deposit amount is recorded in the vault and user transfer accounts.

3. **Withdraw**:

   - The user sends a `Withdraw` instruction.
   - The program calculates 10% of the user's available deposited amount.
   - The calculated amount is transferred from the vault to the user's account.
   - The withdrawal is recorded in the vault and user transfer accounts.
