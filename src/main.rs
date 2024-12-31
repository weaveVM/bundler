pub mod utils;
use crate::utils::evm::generate_random_calldata;
use crate::utils::types::Bundle;
use serde_json;

#[tokio::main]
async fn main() {
    let private_key = String::from("");

    let mut envelopes: Vec<Vec<u8>> = vec![];

    for _ in 0..1 {
        let calldata: String = generate_random_calldata(128_000);

        println!("{:?}", calldata[0..100].to_string());
        let envelope = serde_json::to_vec(&calldata).unwrap();
        envelopes.push(envelope);
    }

    let bundle = Bundle::new()
        .private_key(private_key)
        .envelopes(envelopes)
        .build()
        .propagate()
        .await;
    let _bundle_tx = bundle.unwrap();
}
