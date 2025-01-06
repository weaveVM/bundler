use crate::utils::core::envelope::Envelope;
use crate::utils::core::tx_envelope_writer::TxEnvelopeWrapper;
use crate::utils::errors::Error;
use crate::utils::evm::create_envelope;
use alloy::consensus::TxEnvelope;
use borsh_derive::{BorshDeserialize, BorshSerialize};

#[derive(
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct BundleData {
    pub envelopes: Vec<TxEnvelopeWrapper>,
}

impl BundleData {
    pub fn from(envelopes: Vec<TxEnvelopeWrapper>) -> Self {
        BundleData { envelopes }
    }

    pub async fn create_envelope(
        private_key: Option<&str>,
        envelope: Envelope,
    ) -> Result<TxEnvelope, Error> {
        create_envelope(private_key, envelope).await
    }
}
