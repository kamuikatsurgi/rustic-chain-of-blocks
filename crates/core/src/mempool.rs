use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

const MEMPOOL_JSON: &str = "./mempool.json";

pub type Mempool = Vec<TransactionRequest>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub pk: String,
}

pub fn mempool_init() -> Result<()> {
    let path = Path::new(MEMPOOL_JSON);
    if !path.exists() {
        let mempool: Mempool = vec![];
        let mempool_json = serde_json::to_string_pretty(&mempool)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(mempool_json.as_bytes())?;
    }
    Ok(())
}

pub fn get_all_transaction_reqs() -> Result<Mempool> {
    let path = Path::new(MEMPOOL_JSON);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let mempool: Mempool = serde_json::from_str(&contents)?;
    let empty_mempool = vec![];
    update_mempool(&empty_mempool)?;

    Ok(mempool)
}

pub fn add_transaction_req(from: String, to: String, value: u64, pk: String) -> Result<()> {
    let mut mempool = get_all_transaction_reqs()?;
    let tx_req = TransactionRequest {
        from,
        to,
        value,
        pk,
    };
    mempool.push(tx_req);
    update_mempool(&mempool)?;
    Ok(())
}

pub fn update_mempool(mempool: &Mempool) -> Result<()> {
    let path = Path::new(MEMPOOL_JSON);
    let mempool_json = serde_json::to_string_pretty(mempool)?;
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(mempool_json.as_bytes())?;
    Ok(())
}
