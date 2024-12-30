use alloy::{consensus::TxEnvelope, primitives::Address, providers::{Provider, ProviderBuilder, RootProvider}, transports::http::{Client, Http}};
use eyre::Result;
use rand::{thread_rng, RngCore};
use alloy::{
    network::{TransactionBuilder, EthereumWallet},
    primitives::U256,
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};

use crate::utils::constants::CHAIN_ID;

use futures::future::join_all;
use tokio::task;

use crate::utils::env_var::get_env_key;

use std::io::Cursor;
use serde_json;
use crate::utils::types::{Bundle, TxEnvelopeWrapper};

async fn create_evm_http_client(rpc_url: &str) -> Result<RootProvider<Http<Client>>> {
    let rpc_url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    Ok(provider)
}

pub async fn create_and_sign_legacy_tx(size: usize) -> Result<(TxEnvelope)> {
    // let provider = client
    let signer : PrivateKeySigner = get_env_key("PRIV_KEY".to_string())?.parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    // let nonce = provider.get_transaction_count(signer.clone().address()).await?;
    let rand_input = generate_random_eth_tx_input(size);

    let tx = TransactionRequest::default()
        .with_to("0x0000000000000000000000000000000000000000".parse::<Address>()?)
        .with_nonce(0)
        .with_chain_id(CHAIN_ID)
        .with_input(rand_input.clone())
        .with_value(U256::from(0))
        .with_gas_limit(0)
        .with_gas_price(0);

    let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;
    Ok(tx_envelope)
}

pub async fn broadcast_bundle(envelopes: Vec<u8>, provider: &RootProvider<Http<Client>>, private_key: Option<String>) -> Result<()> {
    let pk = private_key.unwrap_or(get_env_key("PRIV_KEY".to_string())?);
    let signer : PrivateKeySigner = pk.parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    let nonce = provider.get_transaction_count(signer.clone().address()).await?;

    let tx = TransactionRequest::default()
        .with_to("0x0000000000000000000000000000000000000000".parse::<Address>()?)
        .with_nonce(nonce)
        .with_chain_id(provider.get_chain_id().await?)
        .with_input(envelopes)
        .with_value(U256::from(0))
        .with_gas_limit(490_000_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(2_000_000_000);
    let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;
    let tx = provider.send_tx_envelope(tx_envelope).await?;
    println!("{:?}", tx);
    Ok(())
}

fn generate_random_eth_tx_input(byte_len: usize) -> String {
    // Create a buffer with `byte_len` capacity
    let mut buffer = vec![0u8; byte_len];
    // Fill it with random bytes
    thread_rng().fill_bytes(&mut buffer);

    // Encode to hex and prepend `0x`
    let x = format!("0x{}", alloy::hex::encode(buffer));
    println!("{}", &x.len());
    x
}

pub async fn create_bundle(envelope_input_size: usize, envelopes_count: u32, private_key: Option<String>) -> Result<()> {
    let provider = create_evm_http_client("https://testnet-rpc.wvm.dev").await?;
    let provider = std::sync::Arc::new(provider);

    // Create vector of futures
    let futures: Vec<_> = (0..envelopes_count)
        .map(|i| {
            // let provider = provider.clone();
            task::spawn(async move {
                match create_and_sign_legacy_tx(envelope_input_size).await {
                    Ok(tx) => {
                        println!("created tx count {}", i);
                        Ok(TxEnvelopeWrapper::from_envelope(tx))
                    }
                    Err(e) => Err(e),
                }
            })
        })
        .collect();

    // Wait for all futures to complete
    let results = join_all(futures).await;
    let envelopes: Vec<TxEnvelopeWrapper> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    println!("finished creating txs");
    
    let bundle = Bundle::from(envelopes.clone());
    println!("created bundle");

    let serialized = TxEnvelopeWrapper::borsh_ser(&bundle);
    println!("borsh serialized");
    
    let mut input = Cursor::new(&serialized);
    let compressed = TxEnvelopeWrapper::brotli_compress_stream(&mut input);
    println!("brotli compressed");
    
    println!("\nENVELOPES COUNT IN THE BUNDLED TX -- COUNT: {:?} ; INPUT SIZE (BYTE): {:?}", 
        envelopes.len(), envelope_input_size);
    println!("ORIGINAL ENVELOPES BUNDLE SIZE (BYTES): {:?}", 
        serde_json::to_vec(&bundle).unwrap().len());
    println!("FINAL ENVELOPES BUNDLE SIZE (BYTES): {:?}", compressed.len());

    broadcast_bundle(compressed, &provider, private_key).await?;

    Ok(())
}