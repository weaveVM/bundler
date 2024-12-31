pub mod utils;
use std::env;

use crate::utils::evm::generate_random_calldata;
use crate::utils::types::Bundle;
use serde_json;

#[tokio::main]
async fn main() {
    // let private_key = String::from("");

    // let mut envelopes: Vec<Vec<u8>> = vec![];

    // for _ in 0..64 {
    //     let calldata: String = generate_random_calldata(128_000);

    //     println!("{:?}", calldata[0..100].to_string());
    //     let envelope = serde_json::to_vec(&calldata).unwrap();
    //     envelopes.push(envelope);
    // }

    // let bundle = Bundle::new()
    //     .private_key(private_key)
    //     .envelopes(envelopes)
    //     .build()
    //     .propagate()
    //     .await;
    // let _bundle_tx = bundle.unwrap();

    let bundle_txid =
        "0xe81b50f584b9e16267f928f3da94b62fbf1d56f2cddd633142232f26bf894999".to_string();

    let envelopes = Bundle::retrieve_envelopes(bundle_txid).await.unwrap();
    println!("{:#?}", envelopes);
}
