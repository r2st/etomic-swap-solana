use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    hash::Hasher,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::clock::Clock,
    sysvar::{rent::Rent, Sysvar},
};

use crate::instruction::AtomicSwapInstruction;

// Define the state
#[derive(Debug)]
pub struct Payment {
    pub payment_hash: [u8; 32],
    pub lock_time: u64,
    pub state: PaymentState,
}

#[derive(Debug, PartialEq)]
pub enum PaymentState {
    Uninitialized,
    PaymentSent,
    ReceiverSpent,
    SenderRefunded,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::unpack(instruction_data[0], instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        AtomicSwapInstruction::SolanaPayment {
            id,
            receiver,
            secret_hash,
            lock_time,
        } => {
            msg!("Processing SolanaPayment");
        }
    }

    Ok(())
}
