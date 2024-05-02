use base16ct::lower::encode_string;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

const ACCOUNTS_JSON: &str = "./accounts.json";

pub type Accounts = Vec<Account>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

impl Account {
    pub fn get_account_hash(&self) -> Result<String> {
        let hash = Keccak256::new()
            .chain_update(self.address.clone())
            .chain_update(self.balance.to_string().clone())
            .chain_update(self.nonce.to_string().clone())
            .finalize();
        let hash_hex = encode_string(&hash);
        Ok(hash_hex)
    }
}

pub fn accounts_init() -> Result<()> {
    let path = Path::new(ACCOUNTS_JSON);
    if !path.exists() {
        let accounts: Accounts = vec![];
        let accounts_json = serde_json::to_string_pretty(&accounts)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(accounts_json.as_bytes())?;
    }
    Ok(())
}

pub fn update_accounts(account: &Account) -> Result<()> {
    let mut accounts = get_all_accounts()?;
    if let Some(index) = accounts
        .iter()
        .position(|acc| acc.address == account.address)
    {
        accounts[index] = account.clone();
    } else {
        accounts.push(account.clone());
    }
    let path = Path::new(ACCOUNTS_JSON);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    let accounts_json = serde_json::to_string_pretty(&accounts)?;
    file.write_all(accounts_json.as_bytes())?;
    Ok(())
}

pub fn get_all_accounts() -> Result<Accounts> {
    let path = Path::new(ACCOUNTS_JSON);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let accounts: Accounts = serde_json::from_str(&contents)?;
    Ok(accounts)
}

pub fn get_account_by_address(address: &str) -> Result<Account> {
    let accounts = get_all_accounts()?;
    let account = accounts
        .iter()
        .find(|acc| acc.address == address)
        .unwrap()
        .clone();
    Ok(account)
}

pub fn get_state_root() -> Result<String> {
    let accounts = get_all_accounts()?;
    let mut hasher = Keccak256::new();

    if accounts.is_empty() {
        hasher.update(String::default());
        let hash = hasher.finalize();
        let state_root = format!("0x{}", encode_string(&hash));
        return Ok(state_root);
    }

    let account_hashes: Vec<String> = accounts
        .iter()
        .map(|acc| acc.get_account_hash().unwrap())
        .collect();

    for hash in account_hashes {
        hasher.update(hash);
    }

    let hash = hasher.finalize();
    let state_root = format!("0x{}", encode_string(&hash));
    Ok(state_root)
}
