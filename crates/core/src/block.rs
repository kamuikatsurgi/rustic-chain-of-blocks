use crate::{
    account::get_state_root,
    transaction::{get_transactions_root, Transactions},
};
use base16ct::lower::encode_string;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MINERS: [&str; 5] = [
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

impl Header {
    pub fn new(
        parent_hash: String,
        miner: String,
        state_root: String,
        transactions_root: String,
        difficulty: u64,
        total_difficulty: u64,
        number: u64,
        timestamp: u64,
        nonce: u64,
        extra_data: Vec<String>,
    ) -> Self {
        Header {
            parent_hash,
            miner,
            state_root,
            transactions_root,
            difficulty,
            total_difficulty,
            number,
            timestamp,
            nonce,
            extra_data,
        }
    }
}

impl Block {
    pub fn genesis() -> Result<Self> {
        let txs = vec![];

        let parent_hash =
            String::from("0x0000000000000000000000000000000000000000000000000000000000000000");
        let miner = String::from(MINERS[0]);
        let state_root = get_state_root()?;
        let transactions_root = get_transactions_root(&mut txs.clone())?;
        let difficulty: u64 = rand::random::<u8>().into();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let extra_data = vec![];

        let header = Header::new(
            parent_hash,
            miner,
            state_root,
            transactions_root,
            difficulty,
            difficulty,
            0,
            timestamp,
            0,
            extra_data,
        );

        Ok(Block { header, txs })
    }

    pub fn new(
        txs: Transactions,
        parent_hash: String,
        miner: String,
        state_root: String,
        transactions_root: String,
        difficulty: u64,
        total_difficulty: u64,
        number: u64,
        timestamp: u64,
        nonce: u64,
        extra_data: Vec<String>,
    ) -> Self {
        let header = Header::new(
            parent_hash,
            miner,
            state_root,
            transactions_root,
            difficulty,
            total_difficulty,
            number,
            timestamp,
            nonce,
            extra_data,
        );
        Block { txs, header }
    }

    pub fn get_block_hash(&self) -> Result<String> {
        let extra_data_bytes = serde_json::to_string(&self.header.extra_data)?.into_bytes();
        let txs_bytes = serde_json::to_string(&self.txs)?.into_bytes();
        let hash = Keccak256::new()
            .chain_update(self.header.parent_hash.clone())
            .chain_update(self.header.miner.clone())
            .chain_update(self.header.state_root.clone())
            .chain_update(self.header.transactions_root.clone())
            .chain_update(self.header.difficulty.to_string())
            .chain_update(self.header.total_difficulty.to_string())
            .chain_update(self.header.number.to_string())
            .chain_update(self.header.timestamp.to_string())
            .chain_update(self.header.nonce.to_string())
            .chain_update(extra_data_bytes)
            .chain_update(txs_bytes)
            .finalize();
        let hash_hex = format!("0x{}", encode_string(&hash));
        Ok(hash_hex)
    }
}
