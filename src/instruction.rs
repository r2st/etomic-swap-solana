use crate::error_code::{
    INVALID_AMOUNT, INVALID_ATOMIC_SWAP_INSTRUCTION, INVALID_INPUT_LENGTH, INVALID_LOCK_TIME,
    INVALID_RECEIVER_PUBKEY, INVALID_SECRET, INVALID_SECRET_HASH, INVALID_SENDER_PUBKEY,
    INVALID_TOKEN_PROGRAM,
};
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

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
        msg!("input length: {}", input.len());
        match instruction_byte {
            0 => {
                if input.len() != 81 {
                    // 1 + 32 + 8 + + 8 + 32
                    return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_SECRET_HASH))?;

                let lock_time_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_LOCK_TIME))?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let amount_array = input[41..49]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[49..81]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_RECEIVER_PUBKEY))?,
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
                    return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_SECRET_HASH))?;

                let lock_time_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_LOCK_TIME))?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let amount_array = input[41..49]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[49..81]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_RECEIVER_PUBKEY))?,
                );

                let token_program = Pubkey::new_from_array(
                    input[81..113]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_TOKEN_PROGRAM))?,
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
                    return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
                }

                let secret = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_SECRET))?;

                let amount_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let amount = u64::from_le_bytes(amount_array);

                let sender = Pubkey::new_from_array(
                    input[41..73]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_SENDER_PUBKEY))?,
                );

                let token_program = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_TOKEN_PROGRAM))?,
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
                    return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
                }

                let secret_hash = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_SECRET_HASH))?;

                let amount_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let amount = u64::from_le_bytes(amount_array);

                let receiver = Pubkey::new_from_array(
                    input[41..73]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_RECEIVER_PUBKEY))?,
                );

                let token_program = Pubkey::new_from_array(
                    input[73..105]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_TOKEN_PROGRAM))?,
                );

                Ok(AtomicSwapInstruction::SenderRefund {
                    secret_hash,
                    amount,
                    receiver,
                    token_program,
                })
            }
            _ => Err(ProgramError::Custom(INVALID_ATOMIC_SWAP_INSTRUCTION)),
        }
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match *self {
            AtomicSwapInstruction::LamportsPayment {
                ref secret_hash,
                lock_time,
                amount,
                ref receiver,
            } => {
                buf.push(0); // Variant identifier for LamportsPayment
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
            }
            AtomicSwapInstruction::SLPTokenPayment {
                ref secret_hash,
                lock_time,
                amount,
                ref receiver,
                ref token_program,
            } => {
                buf.push(1); // Variant identifier for SLPTokenPayment
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
            }
            AtomicSwapInstruction::ReceiverSpend {
                ref secret,
                amount,
                ref sender,
                ref token_program,
            } => {
                buf.push(2); // Variant identifier for ReceiverSpend
                buf.extend_from_slice(secret);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&sender.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
            }
            AtomicSwapInstruction::SenderRefund {
                ref secret_hash,
                amount,
                ref receiver,
                ref token_program,
            } => {
                buf.push(3); // Variant identifier for SenderRefund
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
            }
        }
        buf
    }
}
