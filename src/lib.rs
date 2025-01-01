pub mod utils;

#[cfg(test)]
mod tests {
    use crate::utils::{
        evm::generate_random_calldata,
        types::{Bundle, Envelope},
    };

    #[tokio::test]
    async fn test_bundle_retrieval() {
        let bundle_txid =
            "0x4bdd031029e2a2072129e70b730e3d16fbccbe2b77803ab3c3adf1aedb89c3e1".to_string();

        let envelopes = Bundle::retrieve_envelopes(bundle_txid).await.unwrap();
        assert_ne!(envelopes.envelopes.len(), 0);
    }

    #[tokio::test]
    async fn test_send_bundle_with_target() {
        // will fail until a tWVM funded EOA (pk) is provided
        let private_key = String::from("");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..1 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope_target = "0xfF67529362D40fB204bD71Dfa636f572f0090C64".to_string();
            let envelope = Envelope::from(envelope_data, Some(envelope_target));
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

    #[tokio::test]
    async fn test_send_bundle_without_target() {
        // will fail until a tWVM funded EOA (pk) is provided, take care about nonce if same wallet is used as in test_send_bundle_with_target
        let private_key = String::from("");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..2 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope = Envelope::from(envelope_data, None);
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
