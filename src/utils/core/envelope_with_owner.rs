use crate::utils::constants::CHAIN_ID;
use crate::utils::core::envelope_signature::EnvelopeSignature;
use crate::utils::core::tags::Tag;
use crate::utils::core::tx_envelope_writer::TxEnvelopeWrapper;
use crate::utils::errors::Error;
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
pub struct TxEnvelopeWrapperWithOwner {
    pub chain_id: u64,
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub from: String,
    pub to: String,
    pub value: String,
    pub input: String,
    pub hash: String,
    pub signature: EnvelopeSignature,
    pub tags: Option<Vec<Tag>>,
}

#[derive(
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct BundleDataWithOwner {
    pub envelopes: Vec<TxEnvelopeWrapperWithOwner>,
}

impl TxEnvelopeWrapperWithOwner {
    pub async fn from(envelope: TxEnvelopeWrapper) -> Self {
        Self {
            from: retrieve_envelope_owner(envelope.clone()).await.unwrap(),
            chain_id: envelope.chain_id,
            nonce: envelope.nonce,
            gas_price: envelope.gas_price,
            gas_limit: envelope.gas_limit,
            to: envelope.to,
            value: envelope.value,
            input: envelope.input,
            hash: envelope.hash,
            signature: envelope.signature,
            tags: envelope.tags,
        }
    }
}

impl BundleDataWithOwner {
    pub async fn from(envelopes: Vec<TxEnvelopeWrapperWithOwner>) -> Self {
        Self { envelopes }
    }
}

pub async fn retrieve_envelope_owner(tx: TxEnvelopeWrapper) -> Result<String, Error> {
    let tx = TxEnvelopeWrapper::to_tx_envelope(&tx).unwrap();
    let from = tx
        .recover_signer()
        .map_err(|_| Error::Other("Failed to parse to address".to_string()))?;
    Ok(from.to_checksum(Some(CHAIN_ID)))
}
