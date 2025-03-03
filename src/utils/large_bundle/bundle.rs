use crate::utils::constants::{ADDRESS_BABE2, LB_CHUNK_MAX_SIZE};
use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::bundle_tx_metadata::BundleTxMetadata;
use crate::utils::core::envelope::Envelope;
use crate::utils::core::tags::Tag;
use crate::utils::errors::Error;
use crate::utils::evm::{create_bundle, retrieve_bundle_data, retrieve_bundle_tx};

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
        assert!(data_len >= LB_CHUNK_MAX_SIZE);

        let chunks_count =
            data_len / LB_CHUNK_MAX_SIZE + ((data_len % LB_CHUNK_MAX_SIZE) / LB_CHUNK_MAX_SIZE);
        let mut chunks = Vec::with_capacity(chunks_count as usize);

        for i in 0..chunks_count {
            let start = (i * LB_CHUNK_MAX_SIZE) as usize;
            let end = std::cmp::min((i + 1) * LB_CHUNK_MAX_SIZE, data_len) as usize;
            let data_chunk = data[start..end].to_vec();
            chunks.push(data_chunk);
        }

        println!("chunks count: {}", chunks_count);

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

        let res = LargeBundle {
            data: Some(data),
            private_key: Some(private_key),
            chunks: Some(chunks),
            chunks_receipts: self.chunks_receipts,
            owner_sig: self.owner_sig,
        };

        // println!("{:?}", res);

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
            chunks_receipts.push(chunk_hash.clone());
            println!("propagated chunks: index #{} - hash: {}", chunks_index, chunk_hash);
        }

        self.chunks_receipts = Some(chunks_receipts);

        Ok(self)
    }

    pub async fn finalize(self) -> Result<String, Error> {
        let private_key: String = self.clone().private_key.ok_or(Error::PrivateKeyNeeded)?;
        let chunks_receipts = self.chunks_receipts.ok_or(Error::EnvelopesNeeded)?;
        // reversal code:
        /*
           let chunks_receipts: Vec<String> = serde_json::from_str(
           &String::from_utf8(data)
           .map_err(|e| Error::Other(e.to_string()))?
           ).map_err(|e| Error::Other(e.to_string()))?;

        */

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
        ];
        let receipts_envelope = vec![Envelope::new().data(Some(data)).tags(Some(tags)).build()?];

        let tx = create_bundle(receipts_envelope, private_key.clone())
            .await
            .map_err(|_| Error::BundleNotCreated)?;

        Ok(tx.tx_hash().to_string())
    }

    // pub async fn retrieve_envelopes(bundle_txid: String) -> Result<BundleData, Error> {
    //     let bundle: BundleTxMetadata = retrieve_bundle_tx(bundle_txid)
    //         .await
    //         .map_err(|_| Error::BundleRetrievalProblem)?;
    //     // assert the bundle versioning by checking target address
    //     if bundle.to.to_lowercase() != ADDRESS_BABE2.to_string().to_ascii_lowercase() {
    //         return Err(Error::UnverifiedAddress);
    //     }

    //     let res: BundleData = retrieve_bundle_data(bundle.calldata).await;
    //     Ok(res)
    // }
}
