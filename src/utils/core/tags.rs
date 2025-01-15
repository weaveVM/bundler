use borsh_derive::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Serialize, Deserialize, PartialEq, BorshSerialize, BorshDeserialize, Clone,
)]

pub struct Tag {
    pub name: String,
    pub value: String,
}

impl Tag {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
