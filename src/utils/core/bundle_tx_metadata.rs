use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleTxMetadata {
    pub block_number: String,
    pub block_hash: String,
    pub calldata: String,
    pub to: String,
}

impl BundleTxMetadata {
    pub fn from(block_number: String, block_hash: String, calldata: String, to: String) -> Self {
        BundleTxMetadata {
            block_number,
            block_hash,
            calldata,
            to,
        }
    }
}
