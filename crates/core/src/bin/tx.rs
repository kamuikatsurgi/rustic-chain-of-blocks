use eyre::Result;
use rustic_chain_of_blocks::{account::*, mempool::add_transaction_req};
use std::io::{self, Write};

fn main() -> Result<()> {
    let from = input("Your address: ")?;
    let to = input("Receiver address: ")?;
    let value = input_parse::<u64>("Value: ")?;
    let pk = input("Your private key: ")?;

    let mut sender_account = get_account_by_address(&from)?;

    if sender_account.balance >= value {
        sender_account.balance -= value;
    } else {
        println!("You don't have sufficient funds to make this transaction!");
        return Ok(());
    }

    sender_account.nonce += 1;
    update_accounts(&sender_account)?;

    let mut receiver_account = get_account_by_address(&to)?;
    receiver_account.balance += value;
    update_accounts(&receiver_account)?;

    add_transaction_req(from, to, value, pk)?;

    println!("📥 Your transaction was added successfully to the mempool 📥");

    Ok(())
}

fn input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn input_parse<T: std::str::FromStr>(prompt: &str) -> Result<T> {
    loop {
        match input(prompt)?.parse() {
            Ok(value) => return Ok(value),
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}
