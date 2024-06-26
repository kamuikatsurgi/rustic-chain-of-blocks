use crate::{
    account::get_state_root,
    transaction::{get_transactions_root, Transactions},
};
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};
use base16ct::lower::encode_string;
use ethers::{signers::LocalWallet, types::H256};
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MINERS: [&str; 5] = [
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
    "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
    "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
];

pub const PKS: [&str; 5] = [
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
    "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
    "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
];

pub type Blocks = Vec<Block>;

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Block {
    pub header: Header,
    pub txs: Transactions,
}

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Header {
    pub parent_hash: String,
    pub miner: String,
    pub state_root: String,
    pub transactions_root: String,
    pub number: u64,
    pub timestamp: u64,
    pub extra_data: Vec<String>,
}

impl Header {
    pub fn new(
        parent_hash: String,
        miner: String,
        state_root: String,
        transactions_root: String,
        number: u64,
        timestamp: u64,
        extra_data: Vec<String>,
    ) -> Self {
        Header { parent_hash, miner, state_root, transactions_root, number, timestamp, extra_data }
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
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut out = Vec::<u8>::new();
        parent_hash.encode(&mut out);
        miner.encode(&mut out);
        state_root.encode(&mut out);
        transactions_root.encode(&mut out);
        0u64.encode(&mut out);
        timestamp.encode(&mut out);

        let hash = Keccak256::digest(&out);
        let header_hash = H256::from_slice(&hash);

        let miner_wallet = LocalWallet::from_str(PKS[0])?;
        let signature = miner_wallet.sign_hash(header_hash)?.to_string();

        let extra_data = vec![signature];

        let header = Header::new(
            parent_hash,
            miner,
            state_root,
            transactions_root,
            0,
            timestamp,
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
        number: u64,
        timestamp: u64,
        extra_data: Vec<String>,
    ) -> Self {
        let header = Header::new(
            parent_hash,
            miner,
            state_root,
            transactions_root,
            number,
            timestamp,
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
            .chain_update(self.header.number.to_string())
            .chain_update(self.header.timestamp.to_string())
            .chain_update(extra_data_bytes)
            .chain_update(txs_bytes)
            .finalize();
        let hash_hex = format!("0x{}", encode_string(&hash));
        Ok(hash_hex)
    }
}
