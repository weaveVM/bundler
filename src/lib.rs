pub mod utils;

#[cfg(test)]
mod tests {
    use eyre::Ok;

    use crate::utils::{
        evm::generate_random_calldata,
        types::{Bundle, Envelope},
    };

    #[tokio::test]
    async fn test_bundle_retrieval() {
        let bundle_txid =
            "0xef8864362474f3a3a0c649068b0577bc3b71820720b845babdfdb715f83dbddd".to_string();

        let envelopes = Bundle::retrieve_envelopes(bundle_txid).await.unwrap();
        println!("{:#?}", envelopes);
        assert_ne!(envelopes.envelopes.len(), 0);
    }

    #[tokio::test]
    async fn test_send_bundle_with_target() -> eyre::Result<()> {
        // will fail until a tWVM funded EOA (pk) is provided
        let private_key = String::from("");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..1 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope_target = "0xfF67529362D40fB204bD71Dfa636f572f0090C64".to_string();
            let envelope = Envelope::new()
                .data(Some(envelope_data))
                .target(Some(envelope_target))
                .build()?;
            envelopes.push(envelope);
        }

        let bundle_tx = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .propagate()
            .await?;
        assert_eq!(bundle_tx.len(), 66);
        Ok(())
    }

    #[tokio::test]
    async fn test_send_bundle_without_target() -> eyre::Result<()> {
        // will fail until a tWVM funded EOA (pk) is provided, take care about nonce if same wallet is used as in test_send_bundle_with_target
        let private_key = String::from("");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..2 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            // let envelope = Envelope::from(Some(envelope_data), None);
            let envelope = Envelope::new()
                .data(Some(envelope_data))
                .target(None)
                .build()?;
            envelopes.push(envelope);
        }

        let bundle_tx = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .propagate()
            .await?;
        assert_eq!(bundle_tx.len(), 66);
        Ok(())
    }
}
