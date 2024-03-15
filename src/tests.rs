use crate::etomic_swap::process_instruction;
use crate::instruction::AtomicSwapInstruction;
use solana_program::hash::{Hash, Hasher};
use solana_program_test::{processor, tokio, BanksClient, ProgramTest, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

pub struct InitializeValues {
    program_id: Pubkey,
    system_program: Pubkey,
    banks_client: BanksClient,
    payer: Keypair,
    recent_blockhash: Hash,
    sender_account: Keypair,
    receiver_account: Keypair,
    swap_account: Keypair,
    lamports_initial_balance: u64,
    secret: [u8; 32],
    secret_hash: [u8; 32],
    lock_time: u64,
    amount: u64,
    token_program: Pubkey,
    receiver: Pubkey,
    sender: Pubkey,
    fee: u64,
}

async fn initialize() -> Result<InitializeValues, Box<dyn std::error::Error>> {
    let program_id = Pubkey::new_unique();
    let system_program = solana_program::system_program::id();
    let mut program_test = ProgramTest::new(
        "etomic-swap-solana",
        program_id,
        processor!(process_instruction), // Processor function
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Setup accounts
    let sender_account = Keypair::new();
    let receiver_account = Keypair::new();
    let swap_account = Keypair::new();
    let lamports_initial_balance = 1000000000;

    let transfer_instruction = solana_program::system_instruction::transfer(
        &payer.pubkey(),
        &sender_account.pubkey(),
        lamports_initial_balance,
    );
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let recipient_account = banks_client
        .get_account(sender_account.pubkey())
        .await?
        .expect("account not found");

    let recipient_balance = banks_client.get_balance(sender_account.pubkey()).await?;
    println!(
        "sender_account lamports_initial_balance: {}",
        recipient_balance
    );
    assert_eq!(recipient_account.lamports, lamports_initial_balance);
    assert_eq!(recipient_balance, lamports_initial_balance);

    let transfer_instruction = solana_program::system_instruction::transfer(
        &payer.pubkey(),
        &receiver_account.pubkey(),
        lamports_initial_balance,
    );
    let mut transaction =
        Transaction::new_with_payer(&[transfer_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let recipient_account = banks_client
        .get_account(receiver_account.pubkey())
        .await?
        .expect("account not found");

    let recipient_balance = banks_client.get_balance(receiver_account.pubkey()).await?;
    println!(
        "receiver_account lamports_initial_balance: {}",
        recipient_balance
    );
    assert_eq!(recipient_account.lamports, lamports_initial_balance);
    assert_eq!(recipient_balance, lamports_initial_balance);

    // Calculate the minimum balance to make the swap account rent-exempt
    // for storing 41 bytes of data
    let rent = banks_client.get_rent().await.expect("get rent");
    let minimum_balance = rent.minimum_balance(41);

    // Create a system instruction to transfer the necessary lamports
    // to the swap account for it to be rent-exempt
    let create_account_instruction = system_instruction::create_account(
        &payer.pubkey(),
        &swap_account.pubkey(),
        minimum_balance,
        41,          // Space in bytes for the account data
        &program_id, // The owner program ID
    );

    // Create and sign a transaction for the account creation and funding
    let mut transaction =
        Transaction::new_with_payer(&[create_account_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &swap_account], recent_blockhash);

    // Process the transaction
    banks_client.process_transaction(transaction).await?;

    let assign_instruction = system_instruction::assign(&swap_account.pubkey(), &program_id);

    let mut transaction = Transaction::new_with_payer(&[assign_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &swap_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

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

    Ok(InitializeValues {
        program_id,
        system_program,
        banks_client,
        payer,
        recent_blockhash,
        sender_account,
        receiver_account,
        swap_account,
        lamports_initial_balance,
        secret,
        secret_hash,
        lock_time,
        amount,
        token_program,
        receiver,
        sender,
        fee: 5000,
    })
}

async fn submit_payment() -> Result<InitializeValues, Box<dyn std::error::Error>> {
    let mut values = initialize().await?;
    let sender_account_balance = values
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let swap_account_balance = values
        .banks_client
        .get_balance(values.swap_account.pubkey())
        .await?;
    println!(
        "before submit_payment: sender_account balance: {}",
        sender_account_balance
    );
    println!(
        "before submit_payment: swap_account balance: {}",
        swap_account_balance
    );
    let swap_instruction = AtomicSwapInstruction::LamportsPayment {
        secret_hash: values.secret_hash,
        lock_time: values.lock_time,
        amount: values.amount,
        receiver: values.receiver,
    };
    let data = swap_instruction.pack();
    let instruction = Instruction {
        program_id: values.program_id,
        // Make sure the sender_account is marked as a signer and the swap_account is not
        accounts: vec![
            AccountMeta::new(values.sender_account.pubkey(), true), // Marked as signer
            AccountMeta::new(values.swap_account.pubkey(), false),  // Not a signer
            AccountMeta::new(values.system_program, false), //system_program must be included
        ],
        data, // The packed instruction data expected by your program
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&values.sender_account.pubkey()));

    // Sign the transaction with the sender_account, as it's required to authorize the transfer
    transaction.sign(
        &[&values.sender_account], // Only the sender needs to sign
        values.recent_blockhash,
    );

    // Process the transaction
    values.banks_client.process_transaction(transaction).await?;

    let sender_account_balance_after = values
        .banks_client
        .get_balance(values.sender_account.pubkey())
        .await?;
    let swap_account_balance_after = values
        .banks_client
        .get_balance(values.swap_account.pubkey())
        .await?;
    println!(
        "after submit_payment: sender_account balance: {}",
        sender_account_balance_after
    );
    println!(
        "after submit_payment: swap_account balance: {}",
        swap_account_balance_after
    );
    assert_eq!(
        sender_account_balance_after,
        sender_account_balance - (values.fee + values.amount)
    );
    assert_eq!(
        swap_account_balance_after,
        swap_account_balance + (values.amount)
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
    /*let swap_instruction = AtomicSwapInstruction::SLPTokenPayment{
        secret_hash, lock_time, amount, receiver, token_program,
    };*/
    let swap_instruction = AtomicSwapInstruction::ReceiverSpend {
        secret: values.secret,
        amount: values.amount,
        sender: values.sender,
        token_program: values.token_program,
    };
    /*let swap_instruction = AtomicSwapInstruction::SenderRefund{
        secret_hash, amount, receiver, token_program,
    };*/
    let mut data = swap_instruction.pack();

    values.recent_blockhash = values.banks_client.get_latest_blockhash().await?;
    let receiver_account_pubkey = values.receiver_account.pubkey();
    let seeds = &[b"swap", receiver_account_pubkey.as_ref()];
    let (vault_pda, bump_seed) = Pubkey::find_program_address(seeds, &values.program_id);
    data.push(bump_seed);
    let instruction = Instruction {
        program_id: values.program_id,
        // Make sure the sender_account is marked as a signer and the swap_account is not
        accounts: vec![
            AccountMeta::new(values.swap_account.pubkey(), false), // Not a signer
            AccountMeta::new(values.receiver_account.pubkey(), true), // Marked as signer
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(values.system_program, false), //system_program must be included
        ],
        data, // The packed instruction data expected by your program
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&values.receiver_account.pubkey()));
    // Sign the transaction with the receiver_account, as it's required to authorize the transfer
    transaction.sign(
        &[&values.receiver_account], // Only the receiver_account needs to sign
        values.recent_blockhash,
    );

    // Process the transaction
    values.banks_client.process_transaction(transaction).await?;

    /*let recipient_balance = values.banks_client.get_balance(values.receiver_account.pubkey()).await?;
    msg!("receiver_account balance: {}", recipient_balance);
    let recipient_balance = values.banks_client.get_balance(values.swap_account.pubkey()).await?;
    msg!("swap_account balance: {}", recipient_balance);*/
    //assert_eq!(recipient_account.lamports, lamports_initial_balance);

    Ok(())
}
