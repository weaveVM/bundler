pub mod utils;
use crate::utils::evm::{create_bundle, create_parallel_bundle};

#[tokio::main]
async fn main() {

    println!("Hello, world!");
    create_parallel_bundle(1, 400_000).await.unwrap() // 840_000
}
