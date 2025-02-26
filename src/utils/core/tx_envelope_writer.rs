use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::envelope::Envelope;
use crate::utils::core::envelope_signature::EnvelopeSignature;
use crate::utils::core::tags::Tag;
use crate::utils::errors::Error;
use alloy::consensus::{Signed, Transaction, TxEnvelope, TxLegacy};
use alloy::primitives::{Address, Bytes, U256};
use alloy::signers::Signature;
use borsh::{from_slice, to_vec};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use std::io::{Read, Write};
use std::str::FromStr;

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
    pub tags: Option<Vec<Tag>>,
}

impl TxEnvelopeWrapper {
    pub fn from_envelope(envelope: TxEnvelope, envelope_metadata: Envelope) -> Self {
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
            tags: envelope_metadata.tags,
        }
    }

    pub fn to_tx_envelope(&self) -> Result<TxEnvelope, Error> {
        let to_address =
            if self.to.is_empty() || self.to == "0x0000000000000000000000000000000000000000" {
                None
            } else {
                Some(
                    Address::from_str(&self.to)
                        .map_err(|_| Error::Other("Failed to parse to address".to_string()))?,
                )
            };

        let value = U256::from_str(&self.value)
            .map_err(|_| Error::Other("Failed to parse transaction value".to_string()))?;

        let input_str = if self.input.starts_with("0x") {
            &self.input
        } else {
            return Err(Error::Other("Input data should start with 0x".to_string()));
        };

        let input = Bytes::from_str(input_str)
            .map_err(|_| Error::Other("Failed to parse input data".to_string()))?;

        let r = U256::from_str(&self.signature.r)
            .map_err(|_| Error::Other("Failed to parse signature r value".to_string()))?;
        let s = U256::from_str(&self.signature.s)
            .map_err(|_| Error::Other("Failed to parse signature s value".to_string()))?;
        let recovery_id = self.signature.y_parity;

        let signature = Signature::from_scalars_and_parity(r.into(), s.into(), recovery_id);

        let hash_str = if self.hash.starts_with("0x") {
            &self.hash
        } else {
            return Err(Error::Other("Hash should start with 0x".to_string()));
        };

        let hash = U256::from_str(hash_str)
            .map_err(|_| Error::Other("Failed to parse transaction hash".to_string()))?;

        let tx = TxLegacy {
            chain_id: Some(self.chain_id),
            nonce: self.nonce,
            gas_price: self.gas_price as u128,
            gas_limit: self.gas_limit,
            to: to_address.into(),
            value,
            input,
        };

        let signed_tx = Signed::new_unchecked(tx, signature, hash.into());
        let tx_envelope = TxEnvelope::Legacy(signed_tx);

        Ok(tx_envelope)
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

    pub fn brotli_decompress_stream<R: Read>(reader: &mut R) -> Vec<u8> {
        let mut writer = Vec::new();
        let mut decoder = brotli::Decompressor::new(reader, 65_536);
        std::io::copy(&mut decoder, &mut writer).unwrap();
        writer
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
