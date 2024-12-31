use alloy::{
    consensus::TxEnvelope,
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};
use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::U256,
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::Result;
use rand::{thread_rng, Rng, RngCore};

use crate::utils::constants::{CHAIN_ID, WVM_RPC_URL, ZERO_ADDRESS};

use futures::future::join_all;
use tokio::task;

use crate::utils::env_var::get_env_key;

use crate::utils::types::{BundleData, TxEnvelopeWrapper};
use serde_json;
use std::io::Cursor;

async fn create_evm_http_client(rpc_url: &str) -> Result<RootProvider<Http<Client>>> {
    let rpc_url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    Ok(provider)
}

pub async fn create_envelope(private_key: Option<&str>, input: Vec<u8>) -> Result<TxEnvelope> {
    let signer: PrivateKeySigner = private_key.unwrap().parse()?;
    let wallet = EthereumWallet::from(signer.clone());

    let tx = TransactionRequest::default()
        .with_to(ZERO_ADDRESS.parse::<Address>()?)
        .with_nonce(0)
        .with_chain_id(CHAIN_ID)
        .with_input(input)
        .with_value(U256::from(0))
        .with_gas_limit(0)
        .with_gas_price(0);

    let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;
    Ok(tx_envelope)
}

async fn broadcast_bundle(
    envelopes: Vec<u8>,
    provider: &RootProvider<Http<Client>>,
    private_key: Option<String>,
) -> Result<(alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>)> {
    let signer: PrivateKeySigner = private_key.unwrap().parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    let nonce = provider
        .get_transaction_count(signer.clone().address())
        .await?;

    let tx = TransactionRequest::default()
        .with_to(ZERO_ADDRESS.parse::<Address>()?)
        .with_nonce(nonce)
        .with_chain_id(CHAIN_ID)
        .with_input(envelopes)
        .with_value(U256::from(0))
        .with_gas_limit(490_000_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(2_000_000_000);
    let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;
    let tx: alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum> =
        provider.send_tx_envelope(tx_envelope).await?;
    println!("{:?}", tx);
    Ok(tx)
}

pub async fn create_bundle(
    envelope_inputs: Vec<Vec<u8>>,
    private_key: String,
) -> Result<alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>> {
    let provider = create_evm_http_client(WVM_RPC_URL).await?;
    let provider = std::sync::Arc::new(provider);
    let private_key = private_key.clone();

    // Create vector of futures
    let futures: Vec<_> = envelope_inputs
        .into_iter()
        .enumerate()
        .map(|(i, input)| {
            let pk = private_key.clone();
            task::spawn(async move {
                match create_envelope(Some(&pk), input).await {
                    Ok(tx) => {
                        println!("created tx count {}", i);
                        Ok(TxEnvelopeWrapper::from_envelope(tx))
                    }
                    Err(e) => Err(e),
                }
            })
        })
        .collect();

    // Rest of the function remains the same
    let results = join_all(futures).await;
    let envelopes: Vec<TxEnvelopeWrapper> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    println!("finished creating txs");

    let bundle = BundleData::from(envelopes.clone());
    println!("created bundle");

    let serialized = TxEnvelopeWrapper::borsh_ser(&bundle);
    println!("borsh serialized");

    let mut input = Cursor::new(&serialized);
    let compressed = TxEnvelopeWrapper::brotli_compress_stream(&mut input);
    println!("brotli compressed");

    println!(
        "\nENVELOPES COUNT IN THE BUNDLED TX -- COUNT: {:?}",
        envelopes.len()
    );
    println!(
        "ORIGINAL ENVELOPES BUNDLE SIZE (BYTES): {:?}",
        serde_json::to_vec(&bundle).unwrap().len()
    );
    println!(
        "FINAL ENVELOPES BUNDLE SIZE (BYTES): {:?}",
        compressed.len()
    );

    let tx: alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum> =
        broadcast_bundle(compressed, &provider, Some(private_key)).await?;

    Ok(tx)
}

pub fn generate_random_calldata(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // Ensure minimum length of 10 (0x + 4 bytes function selector)
    let min_length = 10;
    let actual_length = length.max(min_length);

    // Start with 0x prefix
    let mut calldata = String::with_capacity(actual_length);
    calldata.push_str("0x");

    // Generate random hex characters for the remaining length
    for _ in 2..actual_length {
        let random_hex = rng.gen_range(0..16);
        calldata.push_str(&format!("{:x}", random_hex));
    }

    calldata
}
