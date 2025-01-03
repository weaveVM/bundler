use {
    crate::utils::{
        constants::ADDRESS_BABE1,
        evm::{create_bundle, create_envelope, retrieve_bundle_data, retrieve_bundle_tx},
    },
    alloy::consensus::{Transaction, TxEnvelope},
    borsh::{from_slice, to_vec},
    borsh_derive::{BorshDeserialize, BorshSerialize},
    serde::{self, Deserialize, Serialize},
    std::io::{Read, Write},
};

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
    pub envelopes: Option<Vec<Envelope>>,
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

    pub fn envelopes(mut self, envelopes: Vec<Envelope>) -> Self {
        self.envelopes = Some(envelopes);
        self
    }

    pub fn add_envelope(mut self, envelope: Envelope) -> Self {
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

    pub async fn retrieve_envelopes(bundle_txid: String) -> eyre::Result<BundleData> {
        let bundle: BundleTxMetadata = retrieve_bundle_tx(bundle_txid).await?;
        // assert the bundle versioning by checking target address
        assert_eq!(
            bundle.to.to_lowercase(),
            ADDRESS_BABE1.to_string().to_ascii_lowercase()
        );
        let res: BundleData = retrieve_bundle_data(bundle.calldata).await;
        Ok(res)
    }
}

impl BundleData {
    pub fn from(envelopes: Vec<TxEnvelopeWrapper>) -> Self {
        BundleData { envelopes }
    }

    pub async fn create_envelope(
        private_key: Option<&str>,
        envelope: Envelope,
    ) -> eyre::Result<TxEnvelope> {
        create_envelope(private_key, envelope).await
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub data: Option<Vec<u8>>,
    pub target: Option<String>,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            data: None,
            target: None,
        }
    }

    pub fn data(mut self, data: Option<Vec<u8>>) -> Self {
        self.data = data;
        self
    }

    pub fn target(mut self, target: Option<String>) -> Self {
        self.target = target;
        self
    }

    pub fn build(self) -> eyre::Result<Self> {
        let data = self
            .clone()
            .data
            .ok_or_else(|| eyre::eyre!("data field is required"))?;
        assert_ne!(data.len(), 0);
        Ok(Self {
            data: self.data,
            target: self.target,
        })
    }
}
