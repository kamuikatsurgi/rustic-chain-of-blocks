use crate::account::get_account_by_address;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use base16ct::lower::encode_string;
use ethers::{
    core::types::{transaction::eip2718::TypedTransaction, TransactionRequest},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::str::FromStr;

pub type Transactions = Vec<Transaction>;

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub value: u64,
    pub nonce: u64,
    pub v: String,
    pub r: String,
    pub s: String,
}

impl Transaction {
    pub async fn new(from: String, to: String, value: u64, pk: String) -> Result<Self> {
        let nonce = get_account_by_address(&from)?.nonce;
        let (v, r, s) = sign_transaction(&from, &to, value, &pk).await?;

        Ok(Transaction { sender: from, receiver: to, value, nonce, v, r, s })
    }

    pub fn get_transaction_hash(&self) -> Result<String> {
        let hash = Keccak256::new()
            .chain_update(self.sender.clone())
            .chain_update(self.receiver.clone())
            .chain_update(self.value.to_string())
            .chain_update(self.nonce.to_string())
            .chain_update(self.v.clone())
            .chain_update(self.r.clone())
            .chain_update(self.s.clone())
            .finalize();
        let hash_hex = encode_string(&hash);
        Ok(hash_hex)
    }
}

pub async fn sign_transaction(
    from: &str,
    to: &str,
    value: u64,
    pk: &str,
) -> Result<(String, String, String)> {
    let nonce = get_account_by_address(from)?.nonce;
    let from = Address::from_str(from)?;
    let to = Address::from_str(to)?;
    let wallet = LocalWallet::from_str(pk)?;

    let tx = TypedTransaction::Legacy(
        TransactionRequest::new().from(from).to(to).value(value).nonce(nonce),
    );
    let signature = wallet.sign_transaction(&tx).await?;

    let v = signature.v.to_string();
    let r = signature.r.to_string();
    let s = signature.s.to_string();

    Ok((v, r, s))
}

pub fn get_transactions_root(txs: &mut Transactions) -> Result<String> {
    if txs.is_empty() {
        let mut hasher = Keccak256::new();
        hasher.update(String::default());
        let hash = hasher.finalize();
        let root = format!("0x{}", encode_string(&hash));
        return Ok(root);
    }

    if txs.len() % 2 != 0 {
        txs.push(txs[txs.len() - 1].clone());
    }

    let txs_hashes: Vec<String> = txs.iter().map(|tx| tx.get_transaction_hash().unwrap()).collect();

    let root = format!("0x{}", construct_root(txs_hashes)?);

    Ok(root)
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
            let hash = Keccak256::new().chain_update(left).chain_update(right).finalize();
            parent_nodes.push(encode_string(&hash));
        }

        nodes = parent_nodes;
        if nodes.len() == 1 {
            root.clone_from(&nodes[0]);
            break;
        }

        if nodes.len() % 2 != 0 {
            nodes.push(nodes[nodes.len() - 1].clone());
        }

        leaves = nodes;
    }

    Ok(root)
}
