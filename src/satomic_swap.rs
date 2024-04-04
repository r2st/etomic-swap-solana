use crate::instruction::{
    AtomicSwapInstruction, LamportsPaymentParams, ReceiverSpendParams, SPLTokenPaymentParams,
    SenderRefundParams,
};
use crate::swap_functions::SwapFunctions;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AtomicSwapInstruction::unpack(instruction_data)?;

    match instruction {
        AtomicSwapInstruction::LamportsPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            rent_exemption_lamports,
            vault_bump_seed,
            vault_bump_seed_data,
        } => SwapFunctions::lamports_payment(
            program_id,
            accounts,
            LamportsPaymentParams {
                secret_hash,
                lock_time,
                amount,
                receiver,
                rent_exemption_lamports,
                vault_bump_seed,
                vault_bump_seed_data,
            },
        ),
        AtomicSwapInstruction::SPLTokenPayment {
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
            rent_exemption_lamports,
            vault_bump_seed,
            vault_bump_seed_data,
        } => SwapFunctions::spl_token_payment(
            program_id,
            accounts,
            SPLTokenPaymentParams {
                secret_hash,
                lock_time,
                amount,
                receiver,
                token_program,
                rent_exemption_lamports,
                vault_bump_seed,
                vault_bump_seed_data,
            },
        ),
        AtomicSwapInstruction::ReceiverSpend {
            secret,
            lock_time,
            amount,
            sender,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data,
        } => SwapFunctions::receiver_spend(
            program_id,
            accounts,
            ReceiverSpendParams {
                secret,
                lock_time,
                amount,
                sender,
                token_program,
                vault_bump_seed,
                vault_bump_seed_data,
            },
        ),
        AtomicSwapInstruction::SenderRefund {
            secret_hash,
            lock_time,
            amount,
            receiver,
            token_program,
            vault_bump_seed,
            vault_bump_seed_data,
        } => SwapFunctions::sender_refund(
            program_id,
            accounts,
            SenderRefundParams {
                secret_hash,
                lock_time,
                amount,
                receiver,
                token_program,
                vault_bump_seed,
                vault_bump_seed_data,
            },
        ),
    }
}
