pub mod utils;
use crate::utils::evm::{create_bundle};

#[tokio::main]
async fn main() {

    println!("Hello, world!");
    create_bundle(0, 5_000_000, Option::None).await.unwrap() // 840_000
}
