use serde::{Deserialize, Serialize};

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
