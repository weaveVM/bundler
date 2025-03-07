pub mod utils;

#[cfg(test)]
mod tests {

    use crate::utils::constants::{ADDRESS_BABE1, ADDRESS_BABE2};
    use crate::utils::core::bundle::Bundle;
    use crate::utils::core::envelope::Envelope;
    use crate::utils::core::large_bundle::LargeBundle;
    use crate::utils::core::tags::Tag;
    use crate::utils::evm::{generate_random_bytes, generate_random_calldata};
    use std::fs::File;
    use std::io::{self, Read};
    use std::path::Path;

    #[tokio::test]
    async fn test_bundle_retrieval() {
        let bundle_txid =
            "0xfd6cad44bb32fabb9a413c2610943294c7bfb2c80efe8511008b2414e51c7290".to_string();

        let envelopes = Bundle::retrieve_envelopes(bundle_txid, ADDRESS_BABE1)
            .await
            .unwrap();
        // println!("{:?}", envelopes);
        assert_ne!(envelopes.envelopes.len(), 0);
    }

    #[tokio::test]
    async fn test_bundle_retrieval_with_tagged_envelopes() {
        let bundle_txid =
            "0x9bf22d08777d8c291480ec34c578b49cd5be577ad6dbb5836bdc9b11ec18846b".to_string();

        let envelopes = Bundle::retrieve_envelopes(bundle_txid, ADDRESS_BABE1)
            .await
            .unwrap();
        // println!("{:?}", envelopes);
        assert_eq!(
            envelopes.envelopes[0].clone().tags.unwrap()[0].name,
            "Content-Type".to_string()
        );
    }

    #[tokio::test]
    async fn test_send_bundle_with_target() {
        // will fail until a tWVM funded EOA (pk) is provided
        let private_key =
            String::from("6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..1 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope_target = "0xfF67529362D40fB204bD71Dfa636f572f0090C64".to_string();
            let envelope = Envelope::new()
                .data(Some(envelope_data))
                .target(Some(envelope_target))
                .build()
                .unwrap();
            envelopes.push(envelope);
        }

        let bundle_tx = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .expect("REASON")
            .propagate()
            .await
            .unwrap();
        assert_eq!(bundle_tx.len(), 66);
    }

    #[tokio::test]
    async fn test_send_bundle_without_target() {
        // will fail until a tWVM funded EOA (pk) is provided, take care about nonce if same wallet is used as in test_send_bundle_with_target
        let private_key =
            String::from("6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b");

        let mut envelopes: Vec<Envelope> = vec![];

        for _ in 0..2 {
            let random_calldata: String = generate_random_calldata(128_000); // 128 KB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope = Envelope::new()
                .data(Some(envelope_data))
                .target(None)
                .build()
                .unwrap();
            envelopes.push(envelope);
        }

        let bundle_tx = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .unwrap()
            .propagate()
            .await
            .unwrap();
        println!("{:?}", bundle_tx);
        assert_eq!(bundle_tx.len(), 66);
    }

    #[tokio::test]
    async fn test_send_envelope_with_tags() {
        // will fail until a tWVM funded EOA (pk) is provided
        let private_key =
            String::from("6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b");

        let mut envelopes: Vec<Envelope> = vec![];

        let tags = vec![
            Tag::new("Protocol".to_string(), "large-bundle".to_string()),
            Tag::new("chunk".to_string(), "0".to_string()),
        ];

        for _ in 0..1 {
            let random_calldata: String = generate_random_calldata(8_388_608); // 8 MB of random calldata
            let envelope_data = serde_json::to_vec(&random_calldata).unwrap();
            let envelope = Envelope::new()
                .data(Some(envelope_data))
                .target(None)
                .tags(Some(tags.clone()))
                .build()
                .unwrap();
            envelopes.push(envelope);
        }

        let bundle_tx = Bundle::new()
            .private_key(private_key)
            .envelopes(envelopes)
            .build()
            .expect("REASON")
            .propagate()
            .await
            .unwrap();
        println!("LB BUNDLE TXID: {}", bundle_tx);
        assert_eq!(bundle_tx.len(), 66);
    }

    // #[tokio::test]
    // async fn test_send_large_bundle() {

    //     // will fail until a tWVM funded EOA (pk) is provided, take care about nonce if same wallet is used as in test_send_bundle_with_target
    //     let private_key =
    //         String::from("6f142508b4eea641e33cb2a0161221105086a84584c74245ca463a49effea30b");

    //     let file_path = "./large.mp4";

    //     let content_type = "video/mp4".to_string();
    //     // let content_type = "text/plain".to_string();

    //     let mut file = File::open(file_path).unwrap();
    //     let mut bytes = Vec::new();
    //     file.read_to_end(&mut bytes).unwrap();
    //     let data = "w".repeat(8_000_000).as_bytes().to_vec();

    //     // let random_data: Vec<u8> = generate_random_bytes(1 * 8_388_608); // 104MB
    //     let large_bundle = LargeBundle::new()
    //         .data(bytes)
    //         .private_key(private_key)
    //         .content_type(content_type)
    //         .chunk()
    //         .build()
    //         .unwrap()
    //         .super_propagate_chunks()
    //         .await
    //         .unwrap()
    //         .super_finalize()
    //         .await
    //         .unwrap();

    //     println!("{:?}", large_bundle);

    //     assert_eq!(large_bundle.len(), 66);
    // }

    #[tokio::test]
    async fn test_retrieve_large_bundle() {
        let large_bundle = LargeBundle::retrieve_chunks_receipts(
            "0xb58684c24828f8a80205345897afa7aba478c23005e128e4cda037de6b9ca6fd".to_string(),
        )
        .await
        .unwrap();

        let bundle_hash = large_bundle.reconstruct_large_bundle().await.unwrap();

        // println!("LARGE BUNDLE: {:?}", large_bundle);
        assert_ne!(bundle_hash.len(), 0);
    }
}
