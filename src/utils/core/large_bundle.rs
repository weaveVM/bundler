use crate::utils::constants::{
    ADDRESS_BABE1, ADDRESS_BABE2, LB_CHUNK_MAX_SIZE, LB_SAFE_MAX_SIZE_LIMIT, MAX_SAFE_CHUNKS_IN_LB,
};
use crate::utils::core::bundle::Bundle;
use crate::utils::core::bundle_tx_metadata::BundleTxMetadata;
use crate::utils::core::envelope::Envelope;
use crate::utils::core::tags::Tag;
use crate::utils::errors::Error;
use crate::utils::evm::{create_bundle, retrieve_bundle_data, retrieve_bundle_tx};
use eyre::OptionExt;
use futures::{self};

#[derive(Debug, Default, Clone)]
pub struct LargeBundle {
    pub data: Option<Vec<u8>>,
    pub private_key: Option<String>,
    pub owner_sig: Option<Vec<u8>>,
    pub chunks: Option<Vec<Vec<u8>>>,
    pub chunks_receipts: Option<Vec<String>>,
}

impl LargeBundle {
    pub fn new() -> Self {
        LargeBundle {
            data: None,
            private_key: None,
            owner_sig: None,
            chunks: None,
            chunks_receipts: None,
        }
    }

    pub fn private_key(mut self, key: String) -> Self {
        self.private_key = Some(key);
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn add_data_sig(mut self, sig: Vec<u8>) -> Self {
        self.owner_sig = Some(sig);
        self
    }

    // TODO: add self.data max size check
    pub fn chunk(mut self) -> Self {
        let data = self
            .clone()
            .data
            .ok_or(Error::EnvelopesNeeded)
            .unwrap_or_default();
        let data_len = data.len() as u32;

        // data limits safety check: min 4MB - max 1GB
        assert!(data_len >= LB_CHUNK_MAX_SIZE && data_len <= LB_SAFE_MAX_SIZE_LIMIT as u32);

        let chunks_count =
            data_len / LB_CHUNK_MAX_SIZE + ((data_len % LB_CHUNK_MAX_SIZE) / LB_CHUNK_MAX_SIZE);
        let mut chunks = Vec::with_capacity(chunks_count as usize);

        for i in 0..chunks_count {
            let start = (i * LB_CHUNK_MAX_SIZE) as usize;
            let end = std::cmp::min((i + 1) * LB_CHUNK_MAX_SIZE, data_len) as usize;
            let data_chunk = data[start..end].to_vec();
            chunks.push(data_chunk);
        }

        self.chunks = Some(chunks);
        self
    }

    pub fn build(self) -> Result<LargeBundle, Error> {
        let data = self
            .data
            .filter(|e| !e.is_empty())
            .ok_or(Error::EnvelopesNeeded)?;
        let private_key = self
            .private_key
            .filter(|p| !p.is_empty())
            .ok_or(Error::PrivateKeyNeeded)?;

        let chunks = self
            .chunks
            .filter(|c| !c.is_empty())
            .ok_or(Error::EnvelopesNeeded)?;

        // additional check, 256 chunks == 1GB
        assert!(chunks.len() as u32 <= MAX_SAFE_CHUNKS_IN_LB);

        let res = LargeBundle {
            data: Some(data),
            private_key: Some(private_key),
            chunks: Some(chunks),
            chunks_receipts: self.chunks_receipts,
            owner_sig: self.owner_sig,
        };

        Ok(res)
    }

    pub async fn propagate_chunks(mut self) -> Result<Self, Error> {
        let mut chunks_index = 0;
        let chunks = self.clone().chunks.ok_or(Error::EnvelopesNeeded)?;
        let mut chunks_receipts: Vec<String> = Vec::new();
        let private_key: String = self.clone().private_key.ok_or(Error::PrivateKeyNeeded)?;

        for chunk in chunks {
            let tags = vec![(Tag::new("chunk_index".to_string(), chunks_index.to_string()))];
            let envelope = vec![Envelope::new().data(Some(chunk)).tags(Some(tags)).build()?];
            let tx = create_bundle(envelope, private_key.clone())
                .await
                .map_err(|_| Error::BundleNotCreated)?;
            chunks_index += 1;
            let chunk_hash = tx.tx_hash().to_string();
            chunks_receipts.push(chunk_hash.clone().trim_start_matches("0x").to_string());
            // println!(
            //     "propagated chunks: index #{} - hash: {}",
            //     chunks_index, chunk_hash
            // );
        }

        self.chunks_receipts = Some(chunks_receipts);

        Ok(self)
    }

    pub async fn finalize(self) -> Result<String, Error> {
        let private_key: String = self.clone().private_key.ok_or(Error::PrivateKeyNeeded)?;
        let chunks_receipts = self.chunks_receipts.ok_or(Error::EnvelopesNeeded)?;
        // Vec<String> -> stringified Vec<String> (String) -> &[u8]-> Vec<u8>
        let data = serde_json::to_string(&chunks_receipts)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_bytes()
            .to_vec();
        let tags: Vec<Tag> = vec![
            Tag::new("Protocol".to_string(), "Large-Bundle".to_string()),
            Tag::new(
                "chunks_count".to_string(),
                chunks_receipts.len().to_string(),
            ),
            Tag::new("Content-Type".to_string(), "application/json".to_string()),
        ];
        let receipts_envelope = vec![Envelope::new().data(Some(data)).tags(Some(tags)).build()?];

        let tx = create_bundle(receipts_envelope, private_key.clone())
            .await
            .map_err(|_| Error::BundleNotCreated)?;

        Ok(tx.tx_hash().to_string())
    }

    pub async fn retrieve_chunks_receipts(bundle_txid: String) -> Result<LargeBundle, Error> {
        let bundle: BundleTxMetadata = retrieve_bundle_tx(bundle_txid)
            .await
            .map_err(|_| Error::BundleRetrievalProblem)?;
        // assert the bundle versioning by checking target address
        if bundle.to.to_lowercase() != ADDRESS_BABE1.to_string().to_ascii_lowercase() {
            return Err(Error::UnverifiedAddress);
        }

        let large_bundle = retrieve_bundle_data(bundle.calldata).await;
        let chunks_receipts = large_bundle.envelopes.get(0).ok_or_eyre(Error::Other(
            "Error: cannot reconstruct Large Envelope".to_string(),
        ))?;
        let receipts_data = hex::decode(&chunks_receipts.input.trim_start_matches("0x"))
            .map_err(|e| Error::Other(e.to_string()))?;
        let chunks_receipts: Vec<String> = serde_json::from_str(
            &String::from_utf8(receipts_data).map_err(|e| Error::Other(e.to_string()))?,
        )
        .map_err(|e| Error::Other(e.to_string()))?;

        let chunks_receipts_with_prefix: Vec<String> = chunks_receipts
            .into_iter()
            .map(|chunk| format!("0x{}", chunk))
            .collect();

        Ok(Self {
            chunks_receipts: Some(chunks_receipts_with_prefix),
            ..Default::default()
        })
    }

    pub async fn reconstruct_large_bundle(self) -> Result<Vec<u8>, Error> {
        let chunks_receipts = self.chunks_receipts.ok_or_else(|| Error::EnvelopesNeeded)?;

        let receipt_futures = chunks_receipts
            .clone()
            .into_iter()
            .map(|receipt| async move {
                // println!("UNBUNDLING {}", receipt);
                let receipt_bundle =
                    Bundle::retrieve_envelopes(receipt.clone())
                        .await
                        .map_err(|e| {
                            Error::Other(format!(
                                "Failed to retrieve bundle for receipt {}: {}",
                                receipt, e
                            ))
                        })?;
                let receipt_writer = receipt_bundle
                    .envelopes
                    .get(0)
                    .ok_or_else(|| Error::Other("Error: no envelopes found".to_string()))?;
                let receipt_data = hex::decode(&receipt_writer.input.trim_start_matches("0x"))
                    .map_err(|e| Error::Other(e.to_string()))?;
                Ok::<Vec<u8>, Error>(receipt_data)
            });

        let results = futures::future::try_join_all(receipt_futures)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        Ok(results.into_iter().flatten().collect::<Vec<u8>>())
    }
}
