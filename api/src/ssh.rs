use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SignedPayload {
    payload: String,
    author: String,
    signature: String
}

pub fn verify(payload: String, private_key: PathBuf) -> SignedPayload {
    
}
