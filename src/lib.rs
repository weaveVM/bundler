pub mod utils;

#[cfg(test)]
mod tests {
    use crate::utils::{evm::generate_random_calldata, types::Bundle};

    #[tokio::test]
    async fn test_bundle_retrieval() {
        let bundle_txid =
            "0xe81b50f584b9e16267f928f3da94b62fbf1d56f2cddd633142232f26bf894999".to_string();

        let envelopes = Bundle::retrieve_envelopes(bundle_txid).await.unwrap();
        assert_ne!(envelopes.envelopes.len(), 0);
    }

    #[tokio::test]
    async fn test_send_bundle() {
        let private_key = String::from(""); // pass a funded WeaveVM EOA for the test to pass

        let mut envelopes: Vec<Vec<u8>> = vec![];

        for _ in 0..1 {
            let calldata: String = generate_random_calldata(128_000);
            let envelope = serde_json::to_vec(&calldata).unwrap();
            envelopes.push(envelope);
        }

        let bundle = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .propagate()
            .await;
        let bundle_tx = bundle.unwrap();
        assert_eq!(bundle_tx.len(), 66)
    }
}
