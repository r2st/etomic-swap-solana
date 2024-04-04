use crate::error_code::{
    AMOUNT_ZERO, INVALID_OWNER, INVALID_PAYMENT_HASH, INVALID_PAYMENT_STATE, NOT_SUPPORTED,
    RECEIVER_SET_TO_DEFAULT, SENDER_ACCOUNT_NOT_SIGNER, SENDER_ACCOUNT_NOT_WRITABLE,
    SWAP_ACCOUNT_NOT_FOUND, VAULT_PDA_DATA_NOT_WRITABLE, VAULT_PDA_NOT_WRITABLE,
    VAULT_PDA_PROGRAM_NOT_OWNER, WAIT_FOR_LOCK_TIME,
};
use crate::instruction::{
    LamportsPaymentParams, ReceiverSpendParams, SPLTokenPaymentParams, SenderRefundParams,
};
use crate::payment::{Payment, PaymentState};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::hash::{Hash, Hasher};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use solana_program::{system_instruction, system_program};

pub struct SwapFunctions;

impl SwapFunctions {
    fn payment_hash(
        receiver: &Pubkey,
        sender_account: &Pubkey,
        secret_hash: &[u8; 32],
        token_program: &Pubkey,
        amount: u64,
    ) -> Hash {
        let mut hasher = Hasher::default();
        hasher.hash(receiver.as_ref());
        hasher.hash(sender_account.as_ref());
        hasher.hash(secret_hash);
        hasher.hash(token_program.as_ref());
        let amount_bytes = amount.to_le_bytes();
        hasher.hash(&amount_bytes);
        hasher.result()
    }
    fn create_account(
        program_id: &Pubkey,
        sender_account: &AccountInfo,
        vault_pda_data: &AccountInfo,
        account_infos: &[AccountInfo],
        rent_exemption_lamports: u64,
        vault_seeds_data: &[&[u8]],
    ) -> ProgramResult {
        let create_instruction = system_instruction::create_account(
            sender_account.key,
            vault_pda_data.key,
            rent_exemption_lamports,
            41,
            program_id,
        );
        invoke_signed(&create_instruction, account_infos, &[vault_seeds_data])
    }
    fn store_data(vault_pda_data: &AccountInfo, payment: Payment) -> ProgramResult {
        let data = &mut vault_pda_data.try_borrow_mut_data()?;

        let payment_bytes = payment.pack();

        if data.len() < payment_bytes.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        data[..payment_bytes.len()].copy_from_slice(&payment_bytes);
        Ok(())
    }
    fn validate_accounts(
        sender_account: &AccountInfo,
        vault_pda_data: &AccountInfo,
        vault_pda: &AccountInfo,
    ) -> ProgramResult {
        if !sender_account.is_signer {
            return Err(ProgramError::Custom(SENDER_ACCOUNT_NOT_SIGNER));
        }
        if !sender_account.is_writable {
            return Err(ProgramError::Custom(SENDER_ACCOUNT_NOT_WRITABLE));
        }
        if !vault_pda_data.is_writable {
            return Err(ProgramError::Custom(VAULT_PDA_DATA_NOT_WRITABLE));
        }
        if !vault_pda.is_writable {
            return Err(ProgramError::Custom(VAULT_PDA_NOT_WRITABLE));
        }
        if vault_pda.owner != &system_program::ID {
            return Err(ProgramError::Custom(VAULT_PDA_PROGRAM_NOT_OWNER));
        }
        Ok(())
    }
    fn transfer(
        sender_account: &AccountInfo,
        vault_pda: &AccountInfo,
        account_infos: &[AccountInfo],
        amount: u64,
        vault_seeds: &[&[u8]],
    ) -> ProgramResult {
        let transfer_instruction =
            system_instruction::transfer(sender_account.key, vault_pda.key, amount);
        invoke_signed(&transfer_instruction, account_infos, &[vault_seeds])
    }
    pub fn lamports_payment(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        params: LamportsPaymentParams,
    ) -> ProgramResult {
        if params.receiver == Pubkey::default() {
            return Err(ProgramError::Custom(RECEIVER_SET_TO_DEFAULT));
        }
        if params.amount == 0 {
            return Err(ProgramError::Custom(AMOUNT_ZERO));
        }
        let accounts_iter = &mut accounts.iter();
        let sender_account = next_account_info(accounts_iter)?;
        let vault_pda_data = next_account_info(accounts_iter)?;
        let vault_pda = next_account_info(accounts_iter)?;

        SwapFunctions::validate_accounts(sender_account, vault_pda_data, vault_pda)?;

        let vault_seeds: &[&[u8]] = &[
            b"swap",
            &params.lock_time.to_le_bytes()[..],
            &params.secret_hash[..],
            &[params.vault_bump_seed],
        ];
        let vault_seeds_data: &[&[u8]] = &[
            b"swap_data",
            &params.lock_time.to_le_bytes()[..],
            &params.secret_hash[..],
            &[params.vault_bump_seed_data],
        ];

        let payment_hash = SwapFunctions::payment_hash(
            &params.receiver,
            sender_account.key,
            &params.secret_hash,
            &Pubkey::new_from_array([0; 32]),
            params.amount,
        );
        let payment = Payment {
            payment_hash: payment_hash.to_bytes(),
            lock_time: params.lock_time,
            state: PaymentState::PaymentSent,
        };

        SwapFunctions::create_account(
            program_id,
            sender_account,
            vault_pda_data,
            &[sender_account.clone(), vault_pda_data.clone()],
            params.rent_exemption_lamports,
            vault_seeds_data,
        )?;

        SwapFunctions::store_data(vault_pda_data, payment)?;

        SwapFunctions::transfer(
            sender_account,
            vault_pda,
            &[sender_account.clone(), vault_pda.clone()],
            params.amount + params.rent_exemption_lamports,
            vault_seeds,
        )
    }
    pub fn spl_token_payment(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        params: SPLTokenPaymentParams,
    ) -> ProgramResult {
        if params.receiver == Pubkey::default() {
            return Err(ProgramError::Custom(RECEIVER_SET_TO_DEFAULT));
        }
        if params.amount == 0 {
            return Err(ProgramError::Custom(AMOUNT_ZERO));
        }
        let accounts_iter = &mut accounts.iter();
        let sender_account = next_account_info(accounts_iter)?;
        let vault_pda_data = next_account_info(accounts_iter)?;
        let vault_pda = next_account_info(accounts_iter)?;

        SwapFunctions::validate_accounts(sender_account, vault_pda_data, vault_pda)?;

        let vault_seeds_data: &[&[u8]] = &[
            b"swap_data",
            &params.lock_time.to_le_bytes()[..],
            &params.secret_hash[..],
            &[params.vault_bump_seed_data],
        ];

        let payment_hash = SwapFunctions::payment_hash(
            &params.receiver,
            sender_account.key,
            &params.secret_hash,
            &params.token_program,
            params.amount,
        );

        let payment = Payment {
            payment_hash: payment_hash.to_bytes(),
            lock_time: params.lock_time,
            state: PaymentState::PaymentSent,
        };

        SwapFunctions::create_account(
            program_id,
            sender_account,
            vault_pda_data,
            &[sender_account.clone(), vault_pda_data.clone()],
            params.rent_exemption_lamports,
            vault_seeds_data,
        )?;

        SwapFunctions::store_data(vault_pda_data, payment)?;

        Ok(())
    }
    pub fn receiver_spend(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        params: ReceiverSpendParams,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let receiver_account = next_account_info(accounts_iter)?;
        let vault_pda_data = next_account_info(accounts_iter)?;
        let vault_pda = next_account_info(accounts_iter)?;

        SwapFunctions::validate_accounts(receiver_account, vault_pda_data, vault_pda)?;

        if vault_pda_data.owner != program_id {
            return Err(ProgramError::Custom(INVALID_OWNER));
        }

        let mut hasher = Hasher::default();
        hasher.hash(&params.secret);
        let secret_hash = hasher.result();

        let vault_seeds: &[&[u8]] = &[
            b"swap",
            &params.lock_time.to_le_bytes()[..],
            &secret_hash.to_bytes()[..],
            &[params.vault_bump_seed],
        ];

        let payment_hash = SwapFunctions::payment_hash(
            receiver_account.key,
            &params.sender,
            &secret_hash.to_bytes(),
            &params.token_program,
            params.amount,
        );

        let swap_account_data = &mut vault_pda_data
            .try_borrow_mut_data()
            .map_err(|_| ProgramError::Custom(SWAP_ACCOUNT_NOT_FOUND))?;
        let mut swap_payment = Payment::unpack(swap_account_data)?;
        if swap_payment.payment_hash != payment_hash.to_bytes() {
            return Err(ProgramError::Custom(INVALID_PAYMENT_HASH));
        }
        if swap_payment.state != PaymentState::PaymentSent {
            return Err(ProgramError::Custom(INVALID_PAYMENT_STATE));
        }

        swap_payment.state = PaymentState::ReceiverSpent;
        let payment_bytes = swap_payment.pack();

        swap_account_data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

        if params.token_program == Pubkey::new_from_array([0; 32]) {
            SwapFunctions::transfer(
                vault_pda,
                receiver_account,
                &[vault_pda.clone(), receiver_account.clone()],
                params.amount,
                vault_seeds,
            )
        } else {
            Err(ProgramError::Custom(NOT_SUPPORTED))
        }
    }
    pub fn sender_refund(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        params: SenderRefundParams,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let sender_account = next_account_info(accounts_iter)?;
        let vault_pda_data = next_account_info(accounts_iter)?;
        let vault_pda = next_account_info(accounts_iter)?;

        SwapFunctions::validate_accounts(sender_account, vault_pda_data, vault_pda)?;

        let vault_seeds: &[&[u8]] = &[
            b"swap",
            &params.lock_time.to_le_bytes()[..],
            &params.secret_hash[..],
            &[params.vault_bump_seed],
        ];

        if vault_pda_data.owner != program_id {
            return Err(ProgramError::Custom(INVALID_OWNER));
        }

        let payment_hash = SwapFunctions::payment_hash(
            &params.receiver,
            sender_account.key,
            &params.secret_hash,
            &params.token_program,
            params.amount,
        );

        let swap_account_data = &mut vault_pda_data
            .try_borrow_mut_data()
            .map_err(|_| ProgramError::Custom(SWAP_ACCOUNT_NOT_FOUND))?;
        let mut swap_payment = Payment::unpack(swap_account_data)?;

        let clock = Clock::get()?;
        let now = clock.unix_timestamp as u64;

        if swap_payment.payment_hash != payment_hash.to_bytes() {
            return Err(ProgramError::Custom(INVALID_PAYMENT_HASH));
        }
        if swap_payment.state != PaymentState::PaymentSent {
            return Err(ProgramError::Custom(INVALID_PAYMENT_STATE));
        }
        if swap_payment.lock_time >= now {
            return Err(ProgramError::Custom(WAIT_FOR_LOCK_TIME));
        }
        swap_payment.state = PaymentState::SenderRefunded;
        let payment_bytes = swap_payment.pack();

        swap_account_data[..payment_bytes.len()].copy_from_slice(&payment_bytes);

        if params.token_program == Pubkey::new_from_array([0; 32]) {
            SwapFunctions::transfer(
                vault_pda,
                sender_account,
                &[vault_pda.clone(), sender_account.clone()],
                params.amount,
                vault_seeds,
            )
        } else {
            Err(ProgramError::Custom(NOT_SUPPORTED))
        }
    }
}
