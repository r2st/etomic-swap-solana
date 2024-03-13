use solana_program::{
    account_info::{next_account_info, AccountInfo},
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

#[derive(Debug)]
pub enum AtomicSwapInstruction {
    SolanaPayment {
        id: [u8; 32],
        receiver: Pubkey,
        secret_hash: [u8; 32],
        lock_time: u64,
    },
}

impl AtomicSwapInstruction {
    pub fn unpack(
        instruction_byte: u8,
        input: &[u8],
    ) -> Result<AtomicSwapInstruction, ProgramError> {
        match instruction_byte {
            0 => {
                if input.len() != 105 {
                    // 32 bytes for the public key + 8 bytes for the balance
                    return Err(ProgramError::InvalidAccountData);
                }

                // Extract the owner's public key
                let id = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let receiver = Pubkey::new_from_array(
                    input[33..65]
                        .try_into()
                        .expect("slice with incorrect length"),
                );

                let secret_hash = input[65..97]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                // Extract the balance
                let lock_time_array = input[97..105]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                Ok(AtomicSwapInstruction::SolanaPayment {
                    id,
                    receiver,
                    secret_hash,
                    lock_time,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
