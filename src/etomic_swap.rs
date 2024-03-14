use crate::instruction::AtomicSwapInstruction;
use crate::payment::{Payment, PaymentState};
use borsh::{BorshSchema, BorshSerialize};
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
    system_instruction,
    sysvar::clock::Clock,
    sysvar::{rent::Rent, Sysvar},
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::unpack(instruction_data[0], instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        AtomicSwapInstruction::LamportsPayment {
            id,
            secret_hash,
            lock_time,
            amount,
            receiver,
        } => {
            msg!("Processing Payment");
            if receiver == Pubkey::default() {
                return Err(ProgramError::InvalidInstructionData);
            }
            if amount <= 0 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let swap_account = next_account_info(accounts_iter)?;
            let receiver_account = next_account_info(accounts_iter)?;

            let mut hasher = Hasher::default();
            hasher.hash(&receiver.to_bytes());
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&secret_hash);
            let zero_address = Pubkey::default(); // This is a pubkey filled with zeros
            hasher.hash(&zero_address.to_bytes());
            let amount_bytes = amount.to_le_bytes();
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            let payment = Payment {
                payment_hash: payment_hash.to_bytes(),
                lock_time,
                state: PaymentState::PaymentSent,
            };
            let payment_bytes = payment.pack();

            let data = &mut swap_account.try_borrow_mut_data()?;
            // Ensure the account data has enough space
            if data.len() < payment_bytes.len() {
                msg!("Error: Account data buffer too small");
                return Err(ProgramError::AccountDataTooSmall);
            }

            // Store the data
            data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

            // Log the payment event
            msg!("Payment Event: {:?}", payment);
            Ok(())
        }
        AtomicSwapInstruction::SLPTokenPayment {
            id,
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
        } => {
            msg!("Processing Payment");
            if receiver == Pubkey::default() {
                return Err(ProgramError::InvalidInstructionData);
            }
            if amount <= 0 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let swap_account = next_account_info(accounts_iter)?;
            let receiver_account = next_account_info(accounts_iter)?;
            let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            let mut hasher = Hasher::default();
            hasher.hash(&receiver.to_bytes());
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&token_program.to_bytes());
            let amount_bytes = amount.to_le_bytes();
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            let payment = Payment {
                payment_hash: payment_hash.to_bytes(),
                lock_time,
                state: PaymentState::PaymentSent,
            };
            let payment_bytes = payment.pack();

            let data = &mut swap_account.try_borrow_mut_data()?;
            // Ensure the account data has enough space
            if data.len() < payment_bytes.len() {
                msg!("Error: Account data buffer too small");
                return Err(ProgramError::AccountDataTooSmall);
            }

            // Store the data
            data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

            // Log the payment event
            msg!("Payment Event: {:?}", payment);
            Ok(())
        }
        AtomicSwapInstruction::ReceiverSpend {
            id,
            secret,
            amount,
            sender,
            token_program,
        } => {
            msg!("Processing ReceiverSpend");
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let swap_account = next_account_info(accounts_iter)?;
            let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            let mut hasher = Hasher::default();
            hasher.hash(&secret);
            let secret_hash = hasher.result();

            let mut hasher = Hasher::default();
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&sender.to_bytes());
            hasher.hash(&secret_hash.to_bytes());
            hasher.hash(&token_program.to_bytes());
            let amount_bytes = amount.to_le_bytes(); // Assuming `amount` is u64
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            let swap_account_data = swap_account
                .try_borrow_data()
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let mut swap_payment = Payment::unpack(&swap_account_data)?;
            if swap_payment.payment_hash != payment_hash.to_bytes() {
                return Err(ProgramError::InvalidInstructionData);
            }
            if swap_payment.state != PaymentState::PaymentSent {
                return Err(ProgramError::InvalidInstructionData);
            }

            swap_payment.state = PaymentState::ReceiverSpent;
            let payment_bytes = swap_payment.pack();

            let data = &mut swap_account.try_borrow_mut_data()?;
            // Ensure the account data has enough space
            if data.len() < payment_bytes.len() {
                msg!("Error: Account data buffer too small");
                return Err(ProgramError::AccountDataTooSmall);
            }

            // Store the data
            data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
            let zero_address = Pubkey::default(); // This is a pubkey filled with zeros
            if token_program == zero_address {
                // Native SOL transfer
                let transfer_instruction =
                    system_instruction::transfer(swap_account.key, sender_account.key, amount);

                invoke(
                    &transfer_instruction,
                    &[swap_account.clone(), sender_account.clone()],
                )?;
            } /* else {
                  // SPL Token transfer
                  let source_token_account = next_account_info(accounts_iter)?;
                  let destination_token_account = next_account_info(accounts_iter)?;

                  let token_transfer_instruction = spl_transfer(
                      &spl_token::id(),
                      source_token_account.key,
                      destination_token_account.key,
                      swap_account.key, // Owner of the source token account
                      &[&swap_account.key],
                      amount,
                  )?;

                  invoke_signed(
                      &token_transfer_instruction,
                      &[
                          token_program.clone(),
                          source_token_account.clone(),
                          destination_token_account.clone(),
                          swap_account.clone(),
                      ],
                      &[&[&[...]]], // Provide the correct signer seeds
                  )?;
              }*/

            //Disclose the secret
            msg!(
                "Swap account: {:?} , Secret: {:?}",
                swap_account.key,
                secret
            );
            Ok(())
        }
        AtomicSwapInstruction::SenderRefund {
            id,
            secret_hash,
            amount,
            receiver,
            token_program,
        } => {
            msg!("Processing SenderRefund");
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let swap_account = next_account_info(accounts_iter)?;
            let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            let mut hasher = Hasher::default();
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&receiver.to_bytes());
            hasher.hash(&secret_hash);
            hasher.hash(&token_program.to_bytes());
            let amount_bytes = amount.to_le_bytes(); // Assuming `amount` is u64
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            let swap_account_data = swap_account
                .try_borrow_data()
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let mut swap_payment = Payment::unpack(&swap_account_data)?;

            let clock = Clock::get()?;
            let now = clock.unix_timestamp as u64; // Current time as Unix timestamp

            if swap_payment.payment_hash != payment_hash.to_bytes() && now >= swap_payment.lock_time
            {
                return Err(ProgramError::InvalidInstructionData);
            }
            if swap_payment.state != PaymentState::PaymentSent {
                return Err(ProgramError::InvalidInstructionData);
            }

            swap_payment.state = PaymentState::SenderRefunded;
            let payment_bytes = swap_payment.pack();

            let data = &mut swap_account.try_borrow_mut_data()?;
            // Ensure the account data has enough space
            if data.len() < payment_bytes.len() {
                msg!("Error: Account data buffer too small");
                return Err(ProgramError::AccountDataTooSmall);
            }

            // Store the data
            data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
            let zero_address = Pubkey::default(); // This is a pubkey filled with zeros
            if token_program == zero_address {
                // Native SOL transfer
                let transfer_instruction =
                    system_instruction::transfer(swap_account.key, sender_account.key, amount);

                invoke(
                    &transfer_instruction,
                    &[swap_account.clone(), sender_account.clone()],
                )?;
            } /* else {
                  // SPL Token transfer
                  let source_token_account = next_account_info(accounts_iter)?;
                  let destination_token_account = next_account_info(accounts_iter)?;

                  let token_transfer_instruction = spl_transfer(
                      &spl_token::id(),
                      source_token_account.key,
                      destination_token_account.key,
                      swap_account.key, // Owner of the source token account
                      &[&swap_account.key],
                      amount,
                  )?;

                  invoke_signed(
                      &token_transfer_instruction,
                      &[
                          token_program.clone(),
                          source_token_account.clone(),
                          destination_token_account.clone(),
                          swap_account.clone(),
                      ],
                      &[&[&[...]]], // Provide the correct signer seeds
                  )?;
              }*/

            //Disclose the secret
            msg!("Swap account: {:?}", swap_account.key);
            Ok(())
        }
    }
}
