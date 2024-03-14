use borsh::{BorshDeserialize, BorshSerialize};
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Payment {
    pub payment_hash: [u8; 32],
    pub lock_time: u64,
    pub state: PaymentState,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum PaymentState {
    Uninitialized,
    PaymentSent,
    ReceiverSpent,
    SenderRefunded,
}

impl Payment {
    // Deserializes a byte slice into a Payment struct
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() != 41 {
            // 32 bytes for hash, 8 bytes for lock_time, 1 byte for state
            return Err(ProgramError::InvalidAccountData);
        }

        let payment_hash = input[0..32]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let lock_time = u64::from_le_bytes(
            input[32..40]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let state = match input[40] {
            0 => PaymentState::Uninitialized,
            1 => PaymentState::PaymentSent,
            2 => PaymentState::ReceiverSpent,
            3 => PaymentState::SenderRefunded,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Self {
            payment_hash,
            lock_time,
            state,
        })
    }

    // Serializes the Payment struct into a byte vector
    pub fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Payment hash
        bytes.extend_from_slice(&self.payment_hash);

        // Lock time (u64) as little-endian
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        // State as u8
        let state_byte = match self.state {
            PaymentState::Uninitialized => 0,
            PaymentState::PaymentSent => 1,
            PaymentState::ReceiverSpent => 2,
            PaymentState::SenderRefunded => 3,
        };
        bytes.push(state_byte);

        bytes
    }
}
