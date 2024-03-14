use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(Debug)]
pub enum AtomicSwapInstruction {
    LamportsPayment {
        secret_hash: [u8; 32], // SHA-256 hash
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
    },
    SLPTokenPayment {
        secret_hash: [u8; 32], // SHA-256 hash
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
    },
    ReceiverSpend {
        secret: [u8; 32],
        amount: u64,
        sender: Pubkey,
        token_program: Pubkey,
    },
    SenderRefund {
        secret_hash: [u8; 32], // SHA-256 hash
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
    },
}

impl AtomicSwapInstruction {
    pub fn unpack(
        instruction_byte: u8,
        input: &[u8],
    ) -> Result<AtomicSwapInstruction, ProgramError> {
        match instruction_byte {
            0 => {
                if input.len() != 73 {
                    // 1 + 32 + 8 + + 8 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let lock_time_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let amount_array = input[41..49]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[49..81]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(AtomicSwapInstruction::LamportsPayment {
                    secret_hash,
                    lock_time,
                    amount,
                    receiver,
                })
            }
            1 => {
                if input.len() != 113 {
                    // 1 + 32 + 8 + 8 + 32 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let lock_time_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let amount_array = input[41..49]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[49..81]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                let token_program = Pubkey::new_from_array(
                    input[81..113]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(AtomicSwapInstruction::SLPTokenPayment {
                    secret_hash,
                    lock_time,
                    amount,
                    receiver,
                    token_program,
                })
            }
            2 => {
                if input.len() != 105 {
                    // 1 + 32 + 8 + 32 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let secret = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let amount_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let sender = Pubkey::new_from_array(
                    input[41..73]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                let token_program = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(AtomicSwapInstruction::ReceiverSpend {
                    secret,
                    amount,
                    sender,
                    token_program,
                })
            }
            3 => {
                if input.len() != 105 {
                    // 1 + 32 + 8 + 32 + 32
                    return Err(ProgramError::InvalidAccountData);
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let amount_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[41..73]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                let token_program = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(AtomicSwapInstruction::SenderRefund {
                    secret_hash,
                    amount,
                    receiver,
                    token_program,
                })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
