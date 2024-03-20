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
        rent_exemption_lamports: u64,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    SLPTokenPayment {
        secret_hash: [u8; 32], // SHA-256 hash
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
        rent_exemption_lamports: u64,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    ReceiverSpend {
        secret: [u8; 32],
        lock_time: u64,
        amount: u64,
        sender: Pubkey,
        token_program: Pubkey,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
    },
    SenderRefund {
        secret_hash: [u8; 32], // SHA-256 hash
        lock_time: u64,
        amount: u64,
        receiver: Pubkey,
        token_program: Pubkey,
        vault_bump_seed: u8,
        vault_bump_seed_data: u8,
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
                if input.len() != 91 {
                    // 1 + 32 + 8 + + 8 + 32 + 8 + 1 + 1
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

                let rent_exemption_lamports_array = input[81..89]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let rent_exemption_lamports = u64::from_le_bytes(rent_exemption_lamports_array);

                Ok(AtomicSwapInstruction::LamportsPayment {
                    secret_hash,
                    lock_time,
                    amount,
                    receiver,
                    rent_exemption_lamports,
                    vault_bump_seed: input[89],
                    vault_bump_seed_data: input[90],
                })
            }
            1 => {
                if input.len() != 123 {
                    // 1 + 32 + 8 + 8 + 32 + 32 + 8 + 1 + 1
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

                let rent_exemption_lamports_array = input[113..121]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let rent_exemption_lamports = u64::from_le_bytes(rent_exemption_lamports_array);

                Ok(AtomicSwapInstruction::SLPTokenPayment {
                    secret_hash,
                    lock_time,
                    amount,
                    receiver,
                    token_program,
                    rent_exemption_lamports,
                    vault_bump_seed: input[121],
                    vault_bump_seed_data: input[122],
                })
            }
            2 => {
                if input.len() != 115 {
                    // 1 + 32 + 8 + 32 + 32 + 1 + 1
                    return Err(ProgramError::Custom(INVALID_INPUT_LENGTH));
                }

                let secret = input[1..33]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_SECRET))?;

                let lock_time_array = input[33..41]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_LOCK_TIME))?;
                let lock_time = u64::from_le_bytes(lock_time_array);

                let amount_array = input[41..49]
                    .try_into()
                    .map_err(|_| ProgramError::Custom(INVALID_AMOUNT))?;
                let amount = u64::from_le_bytes(amount_array);

                let sender = Pubkey::new_from_array(
                    input[49..81]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_SENDER_PUBKEY))?,
                );

                let token_program = Pubkey::new_from_array(
                    input[81..113]
                        .try_into()
                        .map_err(|_| ProgramError::Custom(INVALID_TOKEN_PROGRAM))?,
                );

                Ok(AtomicSwapInstruction::ReceiverSpend {
                    secret,
                    lock_time,
                    amount,
                    sender,
                    token_program,
                    vault_bump_seed: input[113],
                    vault_bump_seed_data: input[114],
                })
            }
            3 => {
                if input.len() != 115 {
                    // 1 + 32 + 8 + 32 + 32 + 1 + 1
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

                Ok(AtomicSwapInstruction::SenderRefund {
                    secret_hash,
                    lock_time,
                    amount,
                    receiver,
                    token_program,
                    vault_bump_seed: input[113],
                    vault_bump_seed_data: input[114],
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
                rent_exemption_lamports,
                vault_bump_seed,
                vault_bump_seed_data,
            } => {
                buf.push(0); // Variant identifier for LamportsPayment
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
                buf.extend_from_slice(&rent_exemption_lamports.to_le_bytes());
                buf.push(vault_bump_seed);
                buf.push(vault_bump_seed_data);
            }
            AtomicSwapInstruction::SLPTokenPayment {
                ref secret_hash,
                lock_time,
                amount,
                ref receiver,
                ref token_program,
                rent_exemption_lamports,
                vault_bump_seed,
                vault_bump_seed_data,
            } => {
                buf.push(1); // Variant identifier for SLPTokenPayment
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
                buf.extend_from_slice(&rent_exemption_lamports.to_le_bytes());
                buf.push(vault_bump_seed);
                buf.push(vault_bump_seed_data);
            }
            AtomicSwapInstruction::ReceiverSpend {
                ref secret,
                lock_time,
                amount,
                ref sender,
                ref token_program,
                vault_bump_seed,
                vault_bump_seed_data,
            } => {
                buf.push(2); // Variant identifier for ReceiverSpend
                buf.extend_from_slice(secret);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&sender.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
                buf.push(vault_bump_seed);
                buf.push(vault_bump_seed_data);
            }
            AtomicSwapInstruction::SenderRefund {
                ref secret_hash,
                lock_time,
                amount,
                ref receiver,
                ref token_program,
                vault_bump_seed,
                vault_bump_seed_data,
            } => {
                buf.push(3); // Variant identifier for SenderRefund
                buf.extend_from_slice(secret_hash);
                buf.extend_from_slice(&lock_time.to_le_bytes());
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receiver.to_bytes());
                buf.extend_from_slice(&token_program.to_bytes());
                buf.push(vault_bump_seed);
                buf.push(vault_bump_seed_data);
            }
        }
        buf
    }
}
