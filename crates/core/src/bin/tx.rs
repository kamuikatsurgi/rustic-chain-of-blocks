use eyre::Result;
use rustic_chain_of_blocks::mempool::add_transaction_req;
use std::io::{self, Write};

fn main() -> Result<()> {
    let from = input("Your address: ")?;
    let to = input("Receiver address: ")?;
    let value = input_parse::<u64>("Value: ")?;
    let pk = input("Your private key: ")?;

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
