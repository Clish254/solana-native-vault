use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum VaultInstruction {
    Initialize {},
    Deposit { amount: u64 },
    Withdraw {},
}

#[derive(BorshDeserialize)]
struct DepositPayload {
    amount: u64,
}

#[derive(BorshDeserialize)]
struct WithdrawPayload {
    amount: u64,
}

impl VaultInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => Self::Initialize {},
            1 => {
                let payload = DepositPayload::try_from_slice(rest).unwrap();
                Self::Deposit {
                    amount: payload.amount,
                }
            }
            2 => Self::Withdraw {},
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}
