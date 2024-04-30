use crate::transaction::Transactions;
use serde::{Deserialize, Serialize};

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
