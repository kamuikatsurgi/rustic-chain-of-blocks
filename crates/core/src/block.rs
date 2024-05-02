use crate::{account::*, transaction::*};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const MINERS: [&str; 5] = [
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
    "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
    "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
];

pub type Blocks = Vec<Block>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub txs: Transactions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub parent_hash: String,
    pub miner: String,
    pub state_root: String,
    pub transactions_root: String,
    pub difficulty: u64,
    pub total_difficulty: u64,
    pub number: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub extra_data: Vec<String>,
}

impl Block {
    pub fn genesis() -> Result<Self> {
        let difficulty: u64 = rand::random::<u8>().into();
        let txs = vec![];
        let header = Header {
            parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            miner: MINERS[0].to_string(),
            state_root: format!("0x{}", get_state_root()?),
            transactions_root: format!("0x{}", get_transactions_root(&mut txs.clone())?),
            difficulty,
            total_difficulty: difficulty,
            number: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            nonce: 0,
            extra_data: vec![],
        };

        Ok(Block { header, txs })
    }
}
