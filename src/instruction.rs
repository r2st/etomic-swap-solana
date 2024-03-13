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
    Payment {
        id: [u8; 32],
        secret_hash: [u8; 32],
        lock_time: u64,
        receiver: Pubkey,
    },
    ReceiverSpend {
        id: [u8; 32],
        secret: [u8; 32],
        amount: u64,
        sender: Pubkey,
    },
    SenderRefund {
        id: [u8; 32],
        secret_hash: [u8; 32],
        amount: u64,
        receiver: Pubkey,
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
                    // 1 + 32 + 32 + 8 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let id = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let secret_hash = input[33..65]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let lock_time_array = input[65..73]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let receiver = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .expect("slice with incorrect length"),
                );

                Ok(AtomicSwapInstruction::Payment {
                    id,
                    secret_hash,
                    lock_time,
                    receiver,
                })
            }
            1 => {
                if input.len() != 105 {
                    // 1 + 32 + 32 + 8 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let id = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let secret = input[33..65]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let amount_array = input[65..73]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let sender = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .expect("slice with incorrect length"),
                );

                Ok(AtomicSwapInstruction::ReceiverSpend {
                    id,
                    secret,
                    amount,
                    sender,
                })
            }
            2 => {
                if input.len() != 105 {
                    // 1 + 32 + 32 + 8 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let id = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let secret_hash = input[33..65]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let amount_array = input[65..73]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .expect("slice with incorrect length"),
                );

                Ok(AtomicSwapInstruction::SenderRefund {
                    id,
                    secret_hash,
                    amount,
                    receiver,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
