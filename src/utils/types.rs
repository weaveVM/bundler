use alloy::{
    consensus::{Transaction, TxEnvelope},
    primitives::Bytes,
};
use serde;

use borsh::{from_slice, to_vec};
use borsh_derive::{BorshDeserialize, BorshSerialize};

use std::{
    convert::TryFrom,
    env,
    fs::File,
    io::{Read, Write},
};

// use eyre::{Ok, Result};

use crate::utils::evm::{create_bundle, create_envelope};

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
pub struct TxEnvelopeWrapper {
    pub chain_id: u64,
    pub nonce: u64,
    pub gas_price: u128,
    pub gas_limit: u64,
    pub to: String,
    pub value: String,
    pub input: String,
    pub hash: String,
    pub signature: EnvelopeSignature,
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
pub struct BundleData {
    pub envelopes: Vec<TxEnvelopeWrapper>,
}

#[derive(Debug, Default)]
pub struct Bundle {
    pub envelopes: Option<Vec<Vec<u8>>>,
    pub private_key: Option<String>,
}

impl Bundle {
    pub fn new() -> Self {
        Bundle {
            envelopes: None,
            private_key: None,
        }
    }

    pub fn private_key(mut self, key: String) -> Self {
        self.private_key = Some(key);
        self
    }

    pub fn envelopes(mut self, envelopes: Vec<Vec<u8>>) -> Self {
        self.envelopes = Some(envelopes);
        self
    }

    pub fn add_envelope(mut self, envelope: Vec<u8>) -> Self {
        self.envelopes.get_or_insert(Vec::new()).push(envelope);
        self
    }

    pub fn build(self) -> Self {
        assert_ne!(self.envelopes.clone().unwrap().len(), 0);
        assert_ne!(self.private_key.clone().unwrap().len(), 0);

        Bundle {
            envelopes: self.envelopes,
            private_key: self.private_key,
        }
    }
    pub async fn propagate(self) -> eyre::Result<String> {
        let tx = create_bundle(self.envelopes.unwrap(), self.private_key.unwrap()).await?;
        let hash = tx.tx_hash().to_string();
        Ok(hash)
    }
}

impl BundleData {
    pub fn from(envelopes: Vec<TxEnvelopeWrapper>) -> Self {
        BundleData { envelopes }
    }

    pub async fn create_envelope(
        private_key: Option<&str>,
        input: Vec<u8>,
    ) -> eyre::Result<TxEnvelope> {
        create_envelope(private_key, input).await
    }

    // pub async fn propagate_bundle(
    //     envelope_inputs: Vec<Vec<u8>>,
    //     private_key: Option<String>,
    // ) -> Result<()> {
    //     create_bundle(envelope_inputs, private_key.unwrap()).await
    // }
}

impl TxEnvelopeWrapper {
    pub fn from_envelope(envelope: TxEnvelope) -> Self {
        let sig: alloy::signers::Signature = envelope.signature().clone();

        let env_sig = EnvelopeSignature {
            y_parity: sig.v(),
            r: sig.r().to_string(),
            s: sig.s().to_string(),
        };

        TxEnvelopeWrapper {
            chain_id: envelope.chain_id().unwrap(),
            nonce: envelope.nonce(),
            gas_limit: envelope.gas_limit(),
            gas_price: envelope.gas_price().unwrap(),
            to: envelope.to().unwrap().to_string(),
            value: envelope.value().to_string(),
            input: envelope.input().to_string(),
            hash: envelope.tx_hash().to_string(),
            signature: env_sig,
        }
    }

    pub fn brotli_compress_stream<R: Read>(reader: &mut R) -> Vec<u8> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 65_536, 8, 22); // 65536 -- 64 KiB
        let mut buffer = [0u8; 65_536];

        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            writer.write_all(&buffer[..bytes_read]).unwrap();
        }

        writer.into_inner()
    }

    pub fn brotli_compress(input: &[u8]) -> Vec<u8> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 65536, 9, 22);
        writer.write_all(input).unwrap();
        writer.into_inner()
    }

    pub fn brotli_decompress(input: Vec<u8>) -> Vec<u8> {
        let mut decompressed_data = Vec::new();
        let mut decompressor = brotli::Decompressor::new(input.as_slice(), 32_768); // 32_768 -- 32 KiB

        decompressor
            .read_to_end(&mut decompressed_data)
            .expect("Decompression failed");
        decompressed_data
    }
    pub fn borsh_ser(input: &BundleData) -> Vec<u8> {
        to_vec(input).unwrap()
    }
    pub fn borsh_der(input: Vec<u8>) -> BundleData {
        let res: BundleData = from_slice(&input).expect("error deseriliazing the calldata");
        res
    }
}
