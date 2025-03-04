use crate::utils::constants::ADDRESS_BABE1;
use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::bundle_tx_metadata::BundleTxMetadata;
use crate::utils::core::envelope::Envelope;
use crate::utils::errors::Error;
use crate::utils::evm::{create_bundle, retrieve_bundle_data, retrieve_bundle_tx};

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

    pub fn build(self) -> Result<Bundle, Error> {
        let envelopes = self
            .envelopes
            .filter(|e| !e.is_empty())
            .ok_or(Error::EnvelopesNeeded)?;
        let private_key = self
            .private_key
            .filter(|p| !p.is_empty())
            .ok_or(Error::PrivateKeyNeeded)?;

        Ok(Bundle {
            envelopes: Some(envelopes),
            private_key: Some(private_key),
        })
    }
    pub async fn propagate(self) -> Result<String, Error> {
        let envelopes = self.envelopes.ok_or(Error::EnvelopesNeeded)?;
        let private_key = self.private_key.ok_or(Error::PrivateKeyNeeded)?;

        let tx = create_bundle(envelopes, private_key, ADDRESS_BABE1)
            .await
            .map_err(|_| Error::BundleNotCreated)?;
        let hash = tx.tx_hash().to_string();
        Ok(hash)
    }

    pub async fn retrieve_envelopes(
        bundle_txid: String,
        version: &str,
    ) -> Result<BundleData, Error> {
        let bundle: BundleTxMetadata = retrieve_bundle_tx(bundle_txid)
            .await
            .map_err(|_| Error::BundleRetrievalProblem)?;
        // assert the bundle versioning by checking target address
        if bundle.to.to_lowercase() != version.to_ascii_lowercase() {
            return Err(Error::UnverifiedAddress);
        }

        let res: BundleData = retrieve_bundle_data(bundle.calldata).await;
        Ok(res)
    }
}
