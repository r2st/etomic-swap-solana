use crate::etomic_swap::process_instruction;
use crate::instruction::AtomicSwapInstruction;
use solana_program::hash::Hasher;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub struct InitializeValues {
    program_id: Pubkey,
    system_program: Pubkey,
    context: ProgramTestContext,
    sender_account: Keypair,
    receiver_account: Keypair,
    lamports_initial_balance: u64,
    rent_exemption_lamports: u64,
    secret: [u8; 32],
    secret_hash: [u8; 32],
    lock_time: u64,
    amount: u64,
    token_program: Pubkey,
    receiver: Pubkey,
    sender: Pubkey,
    vault_pda: Pubkey,
    vault_bump_seed: u8,
    vault_pda_data: Pubkey,
    vault_bump_seed_data: u8,
    fee: u64,
}

async fn initialize() -> Result<InitializeValues, Box<dyn std::error::Error>> {
    let program_id = Pubkey::new_unique();
    let system_program = solana_program::system_program::id();
    let program_test = ProgramTest::new(
        "etomic-swap-solana",
        program_id,
        processor!(process_instruction), // Processor function
    );

    let mut context = program_test.start_with_context().await;

    // Setup accounts
    let sender_account = Keypair::new();
    let receiver_account = Keypair::new();
    let lamports_initial_balance = 1000000000;

    let transfer_instruction = solana_program::system_instruction::transfer(
        &context.payer.pubkey(),
        &sender_account.pubkey(),
        lamports_initial_balance,
    );
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&context.payer.pubkey()));
    transaction.sign(&[&context.payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let recipient_account = context
        .banks_client
        .get_account(sender_account.pubkey())
        .await?
        .expect("account not found");

    let recipient_balance = context
        .banks_client
        .get_balance(sender_account.pubkey())
        .await?;
    println!(
        "sender_account lamports_initial_balance: {}",
        recipient_balance
    );
    assert_eq!(recipient_account.lamports, lamports_initial_balance);
    assert_eq!(recipient_balance, lamports_initial_balance);

    let transfer_instruction = solana_program::system_instruction::transfer(
        &context.payer.pubkey(),
        &receiver_account.pubkey(),
        lamports_initial_balance,
    );
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&context.payer.pubkey()));
    transaction.sign(&[&context.payer], context.last_blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let recipient_account = context
        .banks_client
        .get_account(receiver_account.pubkey())
        .await?
        .expect("account not found");

    let recipient_balance = context
        .banks_client
        .get_balance(receiver_account.pubkey())
        .await?;
    println!(
        "receiver_account lamports_initial_balance: {}",
        recipient_balance
    );
    assert_eq!(recipient_account.lamports, lamports_initial_balance);
    assert_eq!(recipient_balance, lamports_initial_balance);

    // Calculate the minimum balance to make the swap account rent-exempt
    // for storing 41 bytes of data
    let rent = context.banks_client.get_rent().await.expect("get rent");
    let rent_exemption_lamports = rent.minimum_balance(41);

    let secret = [0u8; 32];
    let mut hasher = Hasher::default();
    hasher.hash(&secret);
    let secret_hash = hasher.result();
    let secret_hash = secret_hash.to_bytes();
    let lock_time: u64 = 1;
    let amount: u64 = 10000;
    let token_program = Pubkey::new_from_array([0; 32]);
    let receiver = receiver_account.pubkey();
    let sender = sender_account.pubkey();

    let vault_seeds: &[&[u8]] = &[b"swap", &lock_time.to_le_bytes()[..], &secret_hash[..]];
    let vault_seeds_data: &[&[u8]] =
        &[b"swap_data", &lock_time.to_le_bytes()[..], &secret_hash[..]];
    let (vault_pda, vault_bump_seed) = Pubkey::find_program_address(vault_seeds, &program_id);
    let (vault_pda_data, vault_bump_seed_data) =
        Pubkey::find_program_address(vault_seeds_data, &program_id);

    Ok(InitializeValues {
        program_id,
        system_program,
        context,
        sender_account,
        receiver_account,
        lamports_initial_balance,
        rent_exemption_lamports,
        secret,
        secret_hash,
        lock_time,
        amount,
        token_program,
        receiver,
        sender,
        vault_pda,
        vault_bump_seed,
        vault_pda_data,
        vault_bump_seed_data,
        fee: 5000,
    })
}

async fn submit_payment() -> Result<InitializeValues, Box<dyn std::error::Error>> {
    let mut values = initialize().await?;
    let sender_account_balance = values
        .context
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let vault_pda_balance = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "before submit_payment: sender_account balance: {}",
        sender_account_balance
    );
    println!(
        "before submit_payment: vault_pda balance: {}",
        vault_pda_balance
    );
    let swap_instruction = AtomicSwapInstruction::LamportsPayment {
        secret_hash: values.secret_hash,
        lock_time: values.lock_time,
        amount: values.amount,
        receiver: values.receiver,
        rent_exemption_lamports: values.rent_exemption_lamports,
        vault_bump_seed: values.vault_bump_seed,
        vault_bump_seed_data: values.vault_bump_seed_data,
    };
    let data = swap_instruction.pack();
    let instruction = Instruction {
        program_id: values.program_id,
        // Make sure the sender_account is marked as a signer and the vault_pda is not
        accounts: vec![
            AccountMeta::new(values.sender_account.pubkey(), true), // Marked as signer
            AccountMeta::new(values.vault_pda_data, false),         // Not a signer
            AccountMeta::new(values.vault_pda, false),              // Not a signer
            AccountMeta::new(values.system_program, false), //system_program must be included
        ],
        data, // The packed instruction data expected by your program
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&values.sender_account.pubkey()));

    // Sign the transaction with the sender_account, as it's required to authorize the transfer
    transaction.sign(
        &[&values.sender_account], // Only the sender needs to sign
        values.context.last_blockhash,
    );

    // Process the transaction
    values
        .context
        .banks_client
        .process_transaction(transaction)
        .await?;

    let sender_account_balance_after = values
        .context
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let vault_pda_balance_after = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "after submit_payment: sender_account balance: {}",
        sender_account_balance_after
    );
    println!(
        "after submit_payment: vault_pda balance: {}",
        vault_pda_balance_after
    );
    assert_eq!(
        sender_account_balance_after,
        sender_account_balance - (values.fee + values.amount + values.rent_exemption_lamports * 2)
    );
    assert_eq!(
        vault_pda_balance_after,
        vault_pda_balance + values.amount + values.rent_exemption_lamports
    );
    Ok(values)
}

#[tokio::test]
async fn test_submit_payment() -> Result<(), Box<dyn std::error::Error>> {
    let _ = submit_payment().await?;
    Ok(())
}

#[tokio::test]
async fn test_receiver_spend() -> Result<(), Box<dyn std::error::Error>> {
    let mut values = submit_payment().await?;
    let receiver_account_balance = values
        .context
        .banks_client
        .get_balance(values.receiver_account.pubkey())
        .await?;
    let vault_pda_balance = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "before submit_payment: receiver_account balance: {}",
        receiver_account_balance
    );
    println!(
        "before submit_payment: vault_pda balance: {}",
        vault_pda_balance
    );
    /*let swap_instruction = AtomicSwapInstruction::SLPTokenPayment{
        secret_hash, lock_time, amount, receiver, token_program,
    };*/
    let swap_instruction = AtomicSwapInstruction::ReceiverSpend {
        secret: values.secret,
        lock_time: values.lock_time,
        amount: values.amount,
        sender: values.sender,
        token_program: values.token_program,
        vault_bump_seed: values.vault_bump_seed,
        vault_bump_seed_data: values.vault_bump_seed_data,
    };
    /*let swap_instruction = AtomicSwapInstruction::SenderRefund{
        secret_hash, amount, receiver, token_program,
    };*/
    let mut data = swap_instruction.pack();

    values.context.last_blockhash = values.context.banks_client.get_latest_blockhash().await?;
    let instruction = Instruction {
        program_id: values.program_id,
        // Make sure the sender_account is marked as a signer and the vault_pda is not
        accounts: vec![
            AccountMeta::new(values.receiver_account.pubkey(), true), // Marked as signer
            AccountMeta::new(values.vault_pda_data, false),           // Not a signer
            AccountMeta::new(values.vault_pda, false),
            AccountMeta::new(values.system_program, false), //system_program must be included
            AccountMeta::new(values.token_program, false),
        ],
        data, // The packed instruction data expected by your program
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&values.receiver_account.pubkey()));
    // Sign the transaction with the receiver_account, as it's required to authorize the transfer
    transaction.sign(
        &[&values.receiver_account], // Only the receiver_account needs to sign
        values.context.last_blockhash,
    );

    // Process the transaction
    values
        .context
        .banks_client
        .process_transaction(transaction)
        .await?;

    let receiver_account_balance_after = values
        .context
        .banks_client
        .get_balance(values.receiver_account.pubkey())
        .await?;
    let vault_pda_balance_after = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "after submit_payment: receiver_account balance: {}",
        receiver_account_balance_after
    );
    println!(
        "after submit_payment: vault_pda balance: {}",
        vault_pda_balance_after
    );
    assert_eq!(
        receiver_account_balance_after,
        (receiver_account_balance + values.amount) - values.fee
    );
    assert_eq!(vault_pda_balance_after, vault_pda_balance - (values.amount));

    Ok(())
}

#[tokio::test]
async fn test_sender_refund() -> Result<(), Box<dyn std::error::Error>> {
    let mut values = submit_payment().await?;
    let sender_account_balance = values
        .context
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let vault_pda_balance = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "before submit_payment: sender_account balance: {}",
        sender_account_balance
    );
    println!(
        "before submit_payment: vault_pda balance: {}",
        vault_pda_balance
    );
    /*let swap_instruction = AtomicSwapInstruction::SLPTokenPayment{
        secret_hash, lock_time, amount, receiver, token_program,
    };*/
    let swap_instruction = AtomicSwapInstruction::SenderRefund {
        secret_hash: values.secret_hash,
        lock_time: values.lock_time,
        amount: values.amount,
        receiver: values.receiver,
        token_program: values.token_program,
        vault_bump_seed: values.vault_bump_seed,
        vault_bump_seed_data: values.vault_bump_seed_data,
    };
    let mut data = swap_instruction.pack();

    values.context.last_blockhash = values.context.banks_client.get_latest_blockhash().await?;
    let instruction = Instruction {
        program_id: values.program_id,
        // Make sure the sender_account is marked as a signer and the vault_pda is not
        accounts: vec![
            AccountMeta::new(values.sender_account.pubkey(), true), // Marked as signer
            AccountMeta::new(values.vault_pda_data, false),         // Not a signer
            AccountMeta::new(values.vault_pda, false),
            AccountMeta::new(values.system_program, false), //system_program must be included
            AccountMeta::new(values.token_program, false),
        ],
        data, // The packed instruction data expected by your program
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&values.sender_account.pubkey()));
    // Sign the transaction with the receiver_account, as it's required to authorize the transfer
    transaction.sign(
        &[&values.sender_account], // Only the receiver_account needs to sign
        values.context.last_blockhash,
    );

    // Process the transaction
    values
        .context
        .banks_client
        .process_transaction(transaction)
        .await?;

    let sender_account_balance_after = values
        .context
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let vault_pda_balance_after = values
        .context
        .banks_client
        .get_balance(values.vault_pda)
        .await?;
    println!(
        "after submit_payment: receiver_account balance: {}",
        sender_account_balance_after
    );
    println!(
        "after submit_payment: vault_pda balance: {}",
        vault_pda_balance_after
    );
    assert_eq!(
        sender_account_balance_after,
        (sender_account_balance + values.amount) - values.fee
    );
    assert_eq!(vault_pda_balance_after, vault_pda_balance - (values.amount));

    Ok(())
}
