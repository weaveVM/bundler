use borsh_derive::{BorshDeserialize, BorshSerialize};

#[derive(
    Clone,
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct EnvelopeSignature {
    pub y_parity: bool,
    pub r: String,
    pub s: String,
}
