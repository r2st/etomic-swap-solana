use crate::etomic_swap::process_instruction;
use crate::instruction::AtomicSwapInstruction;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

async fn submit_payment() -> Result<(), Box<dyn std::error::Error>> {
    let program_id = Pubkey::new_unique();
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

    let secret = [0; 32];
    let secret_hash = [0; 32];
    let lock_time = 1;
    let amount = 10000;
    let token_program = Pubkey::new_unique();
    let receiver = receiver_account.pubkey();
    let sender = sender_account.pubkey();

    let swap_instruction = AtomicSwapInstruction::LamportsPayment {
        secret_hash,
        lock_time,
        amount,
        receiver,
    };
    /*let swap_instruction = AtomicSwapInstruction::SLPTokenPayment{
        secret_hash, lock_time, amount, receiver, token_program,
    };
    let swap_instruction = AtomicSwapInstruction::ReceiverSpend{
        secret, amount, sender, token_program,
    };
    let swap_instruction = AtomicSwapInstruction::SenderRefund{
        secret_hash, amount, receiver, token_program,
    };*/
    let data = swap_instruction.pack();

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(sender_account.pubkey(), false),
            AccountMeta::new(swap_account.pubkey(), false),
        ],
        data,
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&sender_account.pubkey()));
    transaction.sign(&[&sender_account], recent_blockhash);

    // Process the transaction
    banks_client.process_transaction(transaction).await?;

    Ok(())
}

#[tokio::test]
async fn test_submit_payment() -> Result<(), Box<dyn std::error::Error>> {
    submit_payment().await
}
