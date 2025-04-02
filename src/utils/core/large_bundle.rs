use crate::utils::constants::WVM_RPC_URL;
use crate::utils::constants::{
    ADDRESS_BABE2, LB_CHUNK_MAX_SIZE, LB_SAFE_MAX_SIZE_LIMIT, MAX_SAFE_CHUNKS_IN_LB,
};
use crate::utils::core::bundle::Bundle;
use crate::utils::core::bundle_tx_metadata::BundleTxMetadata;
use crate::utils::core::envelope::Envelope;
use crate::utils::core::super_account::SuperAccount;
use crate::utils::core::tags::Tag;
use crate::utils::errors::Error;
use crate::utils::evm::create_evm_http_client;
use crate::utils::evm::{
    create_bundle, create_bundle_sync, retrieve_bundle_data, retrieve_bundle_tx,
};
use futures::{self};
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct LargeBundle {
    pub data: Option<Vec<u8>>,
    pub private_key: Option<String>,
    pub owner_sig: Option<Vec<u8>>,
    pub chunks: Option<Vec<Vec<u8>>>,
    pub chunks_receipts: Option<Vec<String>>,
    pub content_type: Option<String>,
    pub super_account: Option<SuperAccount>,
    pub chunkers_count: Option<u32>,
}

impl LargeBundle {
    pub fn new() -> Self {
        LargeBundle {
            data: None,
            private_key: None,
            owner_sig: None,
            chunks: None,
            chunks_receipts: None,
            content_type: None,
            super_account: None,
            chunkers_count: None,
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

    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn super_account(mut self, account: SuperAccount) -> Self {
        self.super_account = Some(account);
        self
    }

    pub fn with_chunkers_count(mut self, count: u32) -> Self {
        self.chunkers_count = Some(count);
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

        // data limits safety check: min 1 byte - max 2GB
        assert!(data_len > 0 && data_len <= LB_SAFE_MAX_SIZE_LIMIT as u32);

        let chunks_count =
            data_len / LB_CHUNK_MAX_SIZE + ((data_len % LB_CHUNK_MAX_SIZE) / LB_CHUNK_MAX_SIZE);
        let mut chunks = Vec::with_capacity(chunks_count.max(1) as usize); // ensure at least 1 chunk is counted when data_len < LB_CHUNK_MAX_SIZE

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

        let content_type = self
            .content_type
            .unwrap_or("application/octet-stream".to_string());

        // additional check
        assert!(chunks.len() as u32 <= MAX_SAFE_CHUNKS_IN_LB);

        let res = LargeBundle {
            data: Some(data),
            private_key: Some(private_key),
            chunks: Some(chunks),
            content_type: Some(content_type),
            chunks_receipts: self.chunks_receipts,
            owner_sig: self.owner_sig,
            super_account: self.super_account,
            chunkers_count: self.chunkers_count,
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
            let tx = create_bundle(None, envelope, private_key.clone(), ADDRESS_BABE2)
                .await
                .map_err(|_| Error::BundleNotCreated)?;
            chunks_index += 1;
            let chunk_hash = tx.tx_hash().to_string();
            chunks_receipts.push(chunk_hash.clone().trim_start_matches("0x").to_string());
        }

        self.chunks_receipts = Some(chunks_receipts);

        Ok(self)
    }

    pub async fn finalize(self) -> Result<String, Error> {
        let private_key: String = self.clone().private_key.ok_or(Error::PrivateKeyNeeded)?;
        let chunks_receipts = self.chunks_receipts.ok_or(Error::EnvelopesNeeded)?;
        let http_client = create_evm_http_client(WVM_RPC_URL)
            .await
            .map_err(|err| Error::Other(err.to_string()))?;

        // Vec<String> -> stringified Vec<String> (String) -> &[u8]-> Vec<u8>
        let data = serde_json::to_string(&chunks_receipts)
            .map_err(|e| Error::Other(e.to_string()))?
            .as_bytes()
            .to_vec();

        let tags: Vec<Tag> = vec![
            Tag::new("Protocol".to_string(), "Large-Bundle".to_string()),
            Tag::new(
                "Chunks-Count".to_string(),
                chunks_receipts.len().to_string(),
            ),
            Tag::new("Content-Type".to_string(), "application/json".to_string()),
            Tag::new("Data-Content-Type".to_string(), self.content_type.unwrap()),
        ];

        let receipts_envelope = vec![Envelope::new().data(Some(data)).tags(Some(tags)).build()?];

        let tx = create_bundle_sync(
            Some(http_client),
            receipts_envelope,
            private_key,
            ADDRESS_BABE2,
        )
        .await
        .map_err(|_| Error::BundleNotCreated)?;

        Ok(tx.tx_hash().to_string())
    }

    pub async fn retrieve_chunks_receipts(bundle_txid: String) -> Result<LargeBundle, Error> {
        let bundle: BundleTxMetadata = retrieve_bundle_tx(bundle_txid)
            .await
            .map_err(|_| Error::BundleRetrievalProblem)?;
        // assert the bundle versioning by checking target address
        if bundle.to.to_lowercase() != ADDRESS_BABE2.to_string().to_ascii_lowercase() {
            return Err(Error::UnverifiedAddress);
        }

        let large_bundle = retrieve_bundle_data(bundle.calldata).await;
        let chunks_receipts = large_bundle
            .envelopes
            .get(0)
            .ok_or_else(|| Error::LargeBundleChunksRetrieval)?;

        // retrieve Large Bundle Data-Content-Type
        let data_content_type = chunks_receipts
            .clone()
            .tags
            .unwrap()
            .iter()
            .find(|tag| tag.name.to_lowercase() == "data-content-type")
            .map(|tag| tag.value.clone())
            .unwrap_or_default();

        // retrieve Large Bundle chunk receipts
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
            content_type: Some(data_content_type),
            ..Default::default()
        })
    }

    pub async fn reconstruct_large_bundle(self) -> Result<Vec<u8>, Error> {
        let chunks_receipts = self
            .chunks_receipts
            .ok_or_else(|| Error::LargeBundleChunksRetrieval)?;

        let receipt_futures = chunks_receipts
            .clone()
            .into_iter()
            .map(|receipt| async move {
                let receipt_bundle = Bundle::retrieve_envelopes(receipt.clone(), ADDRESS_BABE2)
                    .await
                    .map_err(|_| Error::LargeBundleReconstruction)?;
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

// SuperAccount method
impl LargeBundle {
    pub async fn super_propagate_chunks(mut self) -> Result<Self, Error> {
        let chunks = self
            .clone()
            .chunks
            .ok_or(Error::LargeBundleChunksRetrieval)?;
        let super_account = self
            .clone()
            .super_account
            .ok_or(Error::SuperAccountNeeded)?;
        let chunkers_count = self.clone().chunkers_count;

        let chunkers = super_account
            .load_chunkers(chunkers_count) // Load all available chunkers
            .await?
            .chunkers
            .ok_or(Error::ChunkersNeeded)?;

        let chunkers_count = chunkers.len();
        println!(
            "Processing {} chunks with {} chunkers",
            chunks.len(),
            chunkers_count
        );
        let mut chunks_receipts: Vec<Option<String>> = vec![None; chunks.len()];
        let chunkers = Arc::new(chunkers);

        let http_client = create_evm_http_client(WVM_RPC_URL)
            .await
            .map_err(|err| Error::Other(err.to_string()))?;
        let max_concurrent = std::cmp::min(chunkers_count, 30);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let (tx, mut rx) =
            tokio::sync::mpsc::channel::<Result<(usize, String), Error>>(chunks.len());

        for (chunk_index, chunk) in chunks.clone().into_iter().enumerate() {
            let chunkers = Arc::clone(&chunkers);
            let semaphore = Arc::clone(&semaphore);
            let http_client = http_client.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                // Determine which chunker to use (round-robin)
                let chunker_index = chunk_index % chunkers_count;
                let chunker = &chunkers[chunker_index];
                let tags = vec![(Tag::new("chunk_index".to_string(), chunk_index.to_string()))];
                let envelope = match Envelope::new().data(Some(chunk)).tags(Some(tags)).build() {
                    Ok(env) => {
                        vec![env]
                    }
                    Err(e) => {
                        println!("Task {} failed to create envelope: {}", chunk_index, e);
                        return;
                    }
                };

                // multiple retry attempts for robustness
                const MAX_RETRIES: usize = 3;
                let mut last_error = None;

                for attempt in 1..=MAX_RETRIES {
                    match create_bundle_sync(
                        Some(http_client.clone()),
                        envelope.clone(),
                        chunker.to_bytes().to_string(),
                        ADDRESS_BABE2,
                    )
                    .await
                    {
                        Ok(tx_result) => {
                            let chunk_hash = tx_result
                                .tx_hash()
                                .to_string()
                                .trim_start_matches("0x")
                                .to_string();
                            let _ = tx.send(Ok((chunk_index, chunk_hash))).await;
                            return;
                        }
                        Err(e) if attempt < MAX_RETRIES => {
                            last_error = Some(e);
                            // Short backoff before retry
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                        Err(e) => {
                            last_error = Some(e);
                            break;
                        }
                    }
                }

                let error = last_error.unwrap_or(Error::BundleNotCreated);
                let _ = tx.send(Err(error)).await;
            });
        }

        // drop the original sender so the channel can close when all tasks complete
        drop(tx);

        // process results as they arrive
        let mut received_count = 0;

        while let Some(result) = rx.recv().await {
            match result {
                Ok((index, hash)) => {
                    chunks_receipts[index] = Some(hash);
                    received_count += 1;
                }
                Err(e) => {
                    println!("Error processing chunk: {}", e);
                    return Err(e);
                }
            }
        }

        let chunks_receipts = chunks_receipts
            .into_iter()
            .enumerate()
            .map(|(i, hash)| {
                hash.ok_or_else(|| Error::Other(format!("Missing receipt for chunk {}", i)))
            })
            .collect::<Result<Vec<String>, Error>>()?;

        self.chunks_receipts = Some(chunks_receipts);

        Ok(self)
    }
}
