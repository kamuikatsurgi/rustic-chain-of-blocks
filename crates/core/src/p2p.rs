use crate::block::Block;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PMessage {
    pub id: u64,
    pub code: Option<u64>,
    pub want: Option<u64>,
    pub data: Option<Vec<u8>>,
    pub random: u64,
}
#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct VoteOnBlock {
    pub block_number: u64,
    pub vote: String,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct NBlocks {
    pub blocks: Vec<Block>,
}
