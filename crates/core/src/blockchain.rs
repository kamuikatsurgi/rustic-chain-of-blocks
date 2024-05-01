use crate::block::{Block, Blocks};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

const BLOCKCHAIN_JSON: &str = "./blockchain.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Blocks,
}

impl Blockchain {
    pub fn init() -> Result<Self> {
        let path = Path::new(BLOCKCHAIN_JSON);
        let mut file = match OpenOptions::new().read(true).open(path) {
            Ok(f) => f,
            Err(_) => {
                let genesis_block = Block::genesis()?;
                let blockchain = Blockchain {
                    blocks: vec![genesis_block.clone()],
                };
                let blockchain_json = serde_json::to_string_pretty(&blockchain)?;
                let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
                file.write_all(blockchain_json.as_bytes())?;
                println!("Mined genesis block 🎉");
                println!("Genesis Block:\n{:#?}", genesis_block);
                return Ok(blockchain);
            }
        };
        println!("Syncing with latest state of blockchain 🔄");
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let blockchain: Blockchain = serde_json::from_str(&content)?;

        Ok(blockchain)
    }
}
