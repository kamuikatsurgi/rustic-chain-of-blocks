use crate::{
    account::get_state_root,
    block::{Block, Blocks, MINERS},
    p2p::NBlocks,
    transaction::{get_transactions_root, Transactions},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const BLOCKCHAIN_JSON: &str = "./blockchain.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Blocks,
}

impl Blockchain {
    pub fn init() -> Result<Self> {
        let path = Path::new(BLOCKCHAIN_JSON);

        if !path.exists() {
            let genesis_block = Block::genesis()?;
            let blockchain = Blockchain { blocks: vec![genesis_block.clone()] };
            let blockchain_json = serde_json::to_string_pretty(&blockchain)?;
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            file.write_all(blockchain_json.as_bytes())?;

            println!("🎉 Mined genesis block 🎉");
            println!("Genesis Block:\n{:#?}", genesis_block);

            return Ok(blockchain);
        }

        println!("🔄 Syncing with latest state of blockchain 🔄");

        let mut file = OpenOptions::new().read(true).open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let blockchain: Blockchain = serde_json::from_str(&content)?;

        Ok(blockchain)
    }

    pub fn propose_block(&self, txs: Transactions, parent_block: &Block) -> Result<Block> {
        let parent_hash = parent_block.get_block_hash()?;
        let miner = MINERS[(parent_block.header.number as usize) % 5].to_string();
        let state_root = get_state_root()?;
        let transactions_root = get_transactions_root(&mut txs.clone())?;
        let number = parent_block.header.number + 1;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        println!("🎉 Proposed a new block by miner {} 🎉", miner);

        Ok(Block::new(
            txs,
            parent_hash,
            miner,
            state_root,
            transactions_root,
            number,
            timestamp,
            vec![],
        ))
    }

    pub fn commit_block(&mut self, block: Block) -> Result<()> {
        self.blocks.push(block.clone());
        update_blockchain(self)?;
        println!("🎉 Mined a new block 🎉");
        println!("{:#?}", block);

        Ok(())
    }
}

pub fn get_last_block() -> Result<Block> {
    let path = Path::new(BLOCKCHAIN_JSON);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let blockchain: Blockchain = serde_json::from_str(&contents)?;

    Ok(blockchain.blocks.last().unwrap().clone())
}

pub fn get_last_n_blocks(n: usize) -> Result<NBlocks> {
    let path = Path::new(BLOCKCHAIN_JSON);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let blockchain: Blockchain = serde_json::from_str(&contents)?;

    let num_blocks = blockchain.blocks.len();
    let start_index = if num_blocks > n { num_blocks - n } else { 0 };
    let last_n_blocks = blockchain.blocks[start_index..].to_vec();

    let nblocks = NBlocks { blocks: last_n_blocks };

    Ok(nblocks)
}

pub fn update_blockchain(chain: &Blockchain) -> Result<()> {
    let path = Path::new(BLOCKCHAIN_JSON);
    let blockchain_json = serde_json::to_string_pretty(chain)?;
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(blockchain_json.as_bytes())?;

    Ok(())
}
