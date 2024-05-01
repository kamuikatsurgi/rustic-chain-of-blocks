use crate::transaction::{get_transactions_root, Transactions};
use ethers::utils::hex;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
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
    pub fn new(header: Header, txs: Transactions) -> Result<Self> {
        Ok(Block { header, txs })
    }
    pub fn mine_block(
        parent_hash: String,
        miner: String,
        state_root: String,
        transactions_root: String,
        difficulty: u64,
        total_difficulty: u64,
        number: u64,
        extra_data: Vec<String>,
        txs: Transactions,
    ) -> Result<Block> {
        let mut nonce = 0;
        let mut start_time = SystemTime::now();

        loop {
            let timestamp = start_time.duration_since(UNIX_EPOCH)?.as_secs();
            let header = Header {
                parent_hash: parent_hash.clone(),
                miner: miner.clone(),
                state_root: state_root.clone(),
                transactions_root: transactions_root.clone(),
                difficulty,
                total_difficulty,
                number,
                timestamp,
                nonce,
                extra_data: extra_data.clone(),
            };

            let hash = header.get_block_hash()?;
            let hash_bytes = hex::decode(&hash)?;

            if Block::check_difficulty(&hash_bytes, difficulty) {
                return Ok(Block { header, txs });
            }

            if nonce == u64::MAX {
                start_time = SystemTime::now();
                nonce = 0;
            }

            nonce += 1;
        }
    }

    fn check_difficulty(hash: &[u8], difficulty: u64) -> bool {
        let mut count = 0;
        for byte in hash {
            if *byte == 0 {
                count += 8;
            } else {
                let leading_zeros = byte.leading_zeros() as u64;
                count += leading_zeros;
                break;
            }
        }
        count >= difficulty
    }

    pub fn genesis() -> Result<Self> {
        let difficulty: u64 = rand::random::<u8>().into();
        let txs: Transactions = vec![];
        let header = Header {
            parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            miner: MINERS[0].to_string(),
            state_root: String::default(),
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

impl Header {
    pub fn new(
        parent_hash: String,
        miner: String,
        state_root: String,
        transactions_root: String,
        difficulty: u64,
        total_difficulty: u64,
        number: u64,
        nonce: u64,
        timestamp: u64,
        extra_data: Vec<String>,
    ) -> Result<Self> {
        Ok(Header {
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
        })
    }

    pub fn get_block_hash(&self) -> Result<String> {
        let mut hasher = Keccak256::new();
        hasher.update(self.parent_hash.as_bytes());
        hasher.update(self.miner.as_bytes());
        hasher.update(self.state_root.as_bytes());
        hasher.update(self.transactions_root.as_bytes());
        hasher.update(&self.difficulty.to_be_bytes());
        hasher.update(&self.total_difficulty.to_be_bytes());
        hasher.update(&self.number.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        for data in &self.extra_data {
            hasher.update(data.as_bytes());
        }
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);
        Ok(hash_hex)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
