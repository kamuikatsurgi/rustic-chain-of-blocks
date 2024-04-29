use base16ct::lower::encode_string;
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
        let hash_hex = encode_string(&hash);
        Ok(hash_hex)
    }
}

pub fn get_transaction_root(txs: &mut Transactions) -> Result<String> {
    if txs.is_empty() {
        let mut hasher = Keccak256::new();
        hasher.update(String::default());
        let hash = hasher.finalize();
        return Ok(encode_string(&hash));
    }

    if txs.len() % 2 != 0 {
        txs.push(txs[txs.len() - 1].clone());
    }

    let txs_hashes: Vec<String> = txs
        .iter()
        .map(|tx| tx.get_transaction_hash().unwrap())
        .collect();

    Ok(construct_root(txs_hashes)?)
}

pub fn construct_root(leaves: Vec<String>) -> Result<String> {
    let mut root = String::new();
    let mut leaves = leaves.clone();

    while leaves.len() > 1 {
        let mut nodes = leaves.clone();
        let mut parent_nodes = Vec::<String>::new();
        for i in 0..nodes.len() / 2 {
            let index = 2 * i;
            let left = nodes[index].clone();
            let right = nodes[index + 1].clone();
            let hash = Keccak256::new()
                .chain_update(left)
                .chain_update(right)
                .finalize();
            parent_nodes.push(encode_string(&hash));
        }

        nodes = parent_nodes;
        if nodes.len() == 1 {
            root = nodes[0].clone();
            break;
        }

        if nodes.len() % 2 != 0 {
            nodes.push(nodes[nodes.len() - 1].clone());
        }

        leaves = nodes;
    }

    Ok(root)
}
