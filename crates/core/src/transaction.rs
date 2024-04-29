use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

pub type Transactions = Vec<Transaction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub value: u64,
    pub v: String,
    pub r: String,
    pub s: String,
}

impl Transaction {
    pub fn get_transaction_hash(&self) -> Result<String> {
        let hash = Keccak256::new()
            .chain_update(self.sender.clone())
            .chain_update(self.receiver.clone())
            .chain_update(self.value.to_string().clone())
            .chain_update(self.v.clone())
            .chain_update(self.r.clone())
            .chain_update(self.s.clone())
            .finalize();

        // To return the hash as 0x....
        let hash_str = format!("0x{:x}", hash);

        Ok(hash_str)
    }
}
