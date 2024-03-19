use crate::error_code::{
    AMOUNT_ZERO, INVALID_OWNER, INVALID_PAYMENT_HASH, INVALID_PAYMENT_STATE, NOT_SUPPORTED,
    RECEIVER_SET_TO_DEFAULT, SWAP_ACCOUNT_NOT_FOUND,
};
use crate::instruction::AtomicSwapInstruction;
use crate::payment::{Payment, PaymentState};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    hash::Hasher,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, system_program,
    sysvar::clock::Clock,
    sysvar::Sysvar,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::unpack(instruction_data[0], instruction_data)?;

    match instruction {
        AtomicSwapInstruction::LamportsPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            rent_exemption_lamports,
            vault_bump_seed,
            vault_bump_seed_data,
        } => {
            msg!("Processing Payment");
            if receiver == Pubkey::default() {
                return Err(ProgramError::Custom(RECEIVER_SET_TO_DEFAULT));
            }
            if amount <= 0 {
                return Err(ProgramError::Custom(AMOUNT_ZERO));
            }
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let vault_pda_data = next_account_info(accounts_iter)?;
            let vault_pda = next_account_info(accounts_iter)?;
            let system_program_account = next_account_info(accounts_iter)?; // System Program account

            assert!(sender_account.is_signer);
            assert!(vault_pda_data.is_writable);
            assert!(vault_pda.is_writable);
            assert_eq!(vault_pda.owner, &system_program::ID);
            assert!(system_program::check_id(system_program_account.key));

            let vault_seeds: &[&[u8]] = &[b"swap", receiver.as_ref(), &[vault_bump_seed]];
            let vault_seeds_data: &[&[u8]] =
                &[b"swap_data", receiver.as_ref(), &[vault_bump_seed_data]];

            let mut hasher = Hasher::default();
            hasher.hash(&receiver.to_bytes());
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&secret_hash);
            let zero_address = Pubkey::new_from_array([0; 32]); // This is a pubkey filled with zeros
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

            {
                let create_instruction = system_instruction::create_account(
                    &sender_account.key,
                    &vault_pda_data.key,
                    rent_exemption_lamports,
                    41,
                    &program_id,
                );

                let account_infos = vec![
                    sender_account.clone(),         // The destination of the funds
                    vault_pda_data.clone(), // Though owned by the program, included for the CPI
                    system_program_account.clone(), // The System Program
                ];

                let _ = invoke_signed(&create_instruction, &account_infos, &[vault_seeds_data])?;

                let data = &mut vault_pda_data.try_borrow_mut_data()?; //swap_account
                                                                       // Ensure the account data has enough space
                if data.len() < payment_bytes.len() {
                    msg!("Error: Account data buffer too small");
                    return Err(ProgramError::AccountDataTooSmall);
                }

                // Store the data
                data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
            }

            // Native SOL transfer
            let transfer_instruction = system_instruction::transfer(
                sender_account.key,               // From
                vault_pda.key,                    // To
                amount + rent_exemption_lamports, // Amount in lamports
            );

            let account_infos = vec![
                sender_account.clone(),         // The source of the funds, must be a signer
                vault_pda.clone(),              // The destination of the funds
                system_program_account.clone(), // The System Program
            ];

            let _ = invoke_signed(&transfer_instruction, &account_infos, &[vault_seeds])?;
            // Log the payment event
            msg!("Payment Event: {:?}", payment);
            Ok(())
        }
        AtomicSwapInstruction::SLPTokenPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
            rent_exemption_lamports,
            vault_bump_seed,
            vault_bump_seed_data,
        } => {
            msg!("Processing Payment");
            if receiver == Pubkey::default() {
                return Err(ProgramError::Custom(RECEIVER_SET_TO_DEFAULT));
            }
            if amount <= 0 {
                return Err(ProgramError::Custom(AMOUNT_ZERO));
            }
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let vault_pda_data = next_account_info(accounts_iter)?;
            let vault_pda = next_account_info(accounts_iter)?;
            let system_program_account = next_account_info(accounts_iter)?; // System Program account
                                                                            //let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            assert!(sender_account.is_signer);
            assert!(vault_pda_data.is_writable);
            assert!(vault_pda.is_writable);
            assert_eq!(vault_pda.owner, &system_program::ID);
            assert!(system_program::check_id(system_program_account.key));

            let _vault_seeds: &[&[u8]] = &[b"swap", receiver.as_ref(), &[vault_bump_seed]];
            let vault_seeds_data: &[&[u8]] =
                &[b"swap_data", receiver.as_ref(), &[vault_bump_seed_data]];

            let mut hasher = Hasher::default();
            hasher.hash(&receiver.to_bytes());
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&secret_hash);
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

            let create_instruction = system_instruction::create_account(
                &sender_account.key,
                &vault_pda_data.key,
                rent_exemption_lamports,
                41,
                &program_id,
            );

            let account_infos = vec![
                vault_pda_data.clone(), // Though owned by the program, included for the CPI
                sender_account.clone(), // The destination of the funds
                system_program_account.clone(), // The System Program
            ];

            let _ = invoke_signed(&create_instruction, &account_infos, &[vault_seeds_data])?;

            let data = &mut vault_pda_data.try_borrow_mut_data()?; //swap_account
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
            secret,
            amount,
            sender,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data,
        } => {
            msg!("Processing ReceiverSpend");
            let accounts_iter = &mut accounts.iter();
            let receiver_account = next_account_info(accounts_iter)?;
            let vault_pda_data = next_account_info(accounts_iter)?;
            let vault_pda = next_account_info(accounts_iter)?;
            let system_program_account = next_account_info(accounts_iter)?; // System Program account
                                                                            //let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            assert!(receiver_account.is_writable);
            assert!(receiver_account.is_signer);
            assert!(vault_pda_data.is_writable);
            assert!(vault_pda.is_writable);
            //assert_eq!(vault_pda.owner, &system_program::ID);
            assert!(system_program::check_id(system_program_account.key));

            let vault_seeds: &[&[u8]] =
                &[b"swap", receiver_account.key.as_ref(), &[vault_bump_seed]];
            let _vault_seeds_data: &[&[u8]] = &[
                b"swap_data",
                receiver_account.key.as_ref(),
                &[vault_bump_seed_data],
            ];
            /*let expected_vault_pda = Pubkey::create_program_address(vault_seeds, program_id)?;
            assert_eq!(vault_pda.key, &expected_vault_pda);*/

            if vault_pda_data.owner != program_id {
                return Err(ProgramError::Custom(INVALID_OWNER));
            }

            let mut hasher = Hasher::default();
            hasher.hash(&secret);
            let secret_hash = hasher.result();

            let mut hasher = Hasher::default();
            hasher.hash(receiver_account.key.as_ref());
            hasher.hash(&sender.to_bytes());
            hasher.hash(&secret_hash.to_bytes());
            hasher.hash(&token_program.to_bytes());
            let amount_bytes = amount.to_le_bytes(); // Assuming `amount` is u64
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            {
                let swap_account_data = &mut vault_pda_data
                    .try_borrow_mut_data()
                    .map_err(|_| ProgramError::Custom(SWAP_ACCOUNT_NOT_FOUND))?;
                let mut swap_payment = Payment::unpack(&swap_account_data)?;
                if swap_payment.payment_hash != payment_hash.to_bytes() {
                    msg!("swap_account payment_hash: {:?}", swap_payment.payment_hash);
                    msg!("payment_hash: {:?}", payment_hash.to_bytes());
                    return Err(ProgramError::Custom(INVALID_PAYMENT_HASH));
                }
                if swap_payment.state != PaymentState::PaymentSent {
                    return Err(ProgramError::Custom(INVALID_PAYMENT_STATE));
                }

                swap_payment.state = PaymentState::ReceiverSpent;
                let payment_bytes = swap_payment.pack();

                // Ensure the account data has enough space
                if swap_account_data.len() < payment_bytes.len() {
                    msg!("Error: Account data buffer too small");
                    return Err(ProgramError::AccountDataTooSmall);
                }

                // Store the data
                swap_account_data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
            }
            if token_program == Pubkey::new_from_array([0; 32]) {
                // Native SOL transfer
                let transfer_instruction = system_instruction::transfer(
                    vault_pda.key,        // From
                    receiver_account.key, // To
                    amount,               // Amount in lamports
                );

                let account_infos = vec![
                    vault_pda.clone(),              // Though owned by the program, included for the CPI
                    receiver_account.clone(),       // The destination of the funds
                    system_program_account.clone(), // The System Program
                ];

                let _ = invoke_signed(&transfer_instruction, &account_infos, &[vault_seeds])?;
            } else {
                // SPL Token transfer
                msg!("Not Supported: SPL Token transfer");
                return Err(ProgramError::Custom(NOT_SUPPORTED));
                /*let source_token_account = next_account_info(accounts_iter)?;
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
                )?;*/
            }

            //Disclose the secret
            msg!(
                "Swap account: {:?} , Secret: {:?}",
                vault_pda_data.key,
                secret
            );
            Ok(())
        }
        AtomicSwapInstruction::SenderRefund {
            secret_hash,
            amount,
            receiver,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data,
        } => {
            msg!("Processing SenderRefund");
            let accounts_iter = &mut accounts.iter();
            let sender_account = next_account_info(accounts_iter)?;
            let vault_pda_data = next_account_info(accounts_iter)?;
            let vault_pda = next_account_info(accounts_iter)?;
            let system_program_account = next_account_info(accounts_iter)?; // System Program account
                                                                            //let token_program_account = next_account_info(accounts_iter)?; // SPL Token program account

            assert!(sender_account.is_writable);
            assert!(sender_account.is_signer);
            assert!(vault_pda_data.is_writable);
            assert!(vault_pda.is_writable);
            //assert_eq!(vault_pda.owner, &system_program::ID);
            assert!(system_program::check_id(system_program_account.key));

            let vault_seeds: &[&[u8]] = &[b"swap", receiver.as_ref(), &[vault_bump_seed]];
            let _vault_seeds_data: &[&[u8]] =
                &[b"swap_data", receiver.as_ref(), &[vault_bump_seed_data]];
            /*let expected_vault_pda = Pubkey::create_program_address(vault_seeds, program_id)?;
            assert_eq!(vault_pda.key, &expected_vault_pda);*/

            if vault_pda_data.owner != program_id {
                return Err(ProgramError::Custom(INVALID_OWNER));
            }

            let mut hasher = Hasher::default();
            hasher.hash(&receiver.to_bytes());
            hasher.hash(sender_account.key.as_ref());
            hasher.hash(&secret_hash);
            hasher.hash(&token_program.to_bytes());
            let amount_bytes = amount.to_le_bytes(); // Assuming `amount` is u64
            hasher.hash(&amount_bytes);

            let payment_hash = hasher.result();

            {
                let swap_account_data = &mut vault_pda_data
                    .try_borrow_mut_data()
                    .map_err(|_| ProgramError::Custom(SWAP_ACCOUNT_NOT_FOUND))?;
                let mut swap_payment = Payment::unpack(&swap_account_data)?;

                let clock = Clock::get()?;
                let now = clock.unix_timestamp as u64; // Current time as Unix timestamp

                if swap_payment.payment_hash != payment_hash.to_bytes()
                    && now >= swap_payment.lock_time
                {
                    return Err(ProgramError::Custom(INVALID_PAYMENT_HASH));
                }
                if swap_payment.state != PaymentState::PaymentSent {
                    return Err(ProgramError::Custom(INVALID_PAYMENT_STATE));
                }

                swap_payment.state = PaymentState::SenderRefunded;
                let payment_bytes = swap_payment.pack();

                // Ensure the account data has enough space
                if swap_account_data.len() < payment_bytes.len() {
                    msg!("Error: Account data buffer too small");
                    return Err(ProgramError::AccountDataTooSmall);
                }

                // Store the data
                swap_account_data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
            }
            if token_program == Pubkey::new_from_array([0; 32]) {
                // Native SOL transfer
                let transfer_instruction = system_instruction::transfer(
                    vault_pda.key,      // From
                    sender_account.key, // To
                    amount,             // Amount in lamports
                );

                let account_infos = vec![
                    vault_pda.clone(),              // Though owned by the program, included for the CPI
                    sender_account.clone(),         // The destination of the funds
                    system_program_account.clone(), // The System Program
                ];

                let _ = invoke_signed(&transfer_instruction, &account_infos, &[vault_seeds])?;
            } else {
                // SPL Token transfer
                msg!("Not Supported: SPL Token transfer");
                return Err(ProgramError::Custom(NOT_SUPPORTED));
                /*let source_token_account = next_account_info(accounts_iter)?;
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
                )?;*/
            }

            msg!("Swap account: {:?}", vault_pda_data.key);
            Ok(())
        }
    }
}
