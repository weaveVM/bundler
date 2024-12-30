pub mod utils;
use crate::utils::types::Bundle;
use serde_json;

#[tokio::main]
async fn main() {
    let private_key = String::from("");

    println!("Hello, world!");
    let envelope_0: Vec<u8> = serde_json::to_vec("hello world from weaveVM bundler").unwrap();
    let envelope_1: Vec<u8> = vec![1, 2, 3];
    let _bundle = Bundle::propagate_bundle(vec![envelope_0, envelope_1], Some(private_key))
        .await
        .unwrap();
}
