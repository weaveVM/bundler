use {
    crate::utils::{
        constants::{CHAIN_ID, WVM_RPC_URL},
        types::{BundleData, Envelope, GetBlockFromTx, TxEnvelopeWrapper},
    },
    alloy::{
        consensus::TxEnvelope,
        network::{EthereumWallet, TransactionBuilder},
        primitives::{Address, B256, U256},
        providers::{Provider, ProviderBuilder, RootProvider},
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
        transports::http::{Client, Http},
    },
    eyre::Result,
    futures::future::join_all,
    hex,
    rand::Rng,
    serde_json,
    std::str::FromStr,
    tokio::task,
};

async fn create_evm_http_client(rpc_url: &str) -> Result<RootProvider<Http<Client>>> {
    let rpc_url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    Ok(provider)
}

pub async fn create_envelope(private_key: Option<&str>, envelope: Envelope) -> Result<TxEnvelope> {
    let signer: PrivateKeySigner = private_key.unwrap().parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    let target_address = envelope
        .target
        .map(|t| t.parse::<Address>().unwrap_or(Address::ZERO))
        .unwrap_or(Address::ZERO);

    let tx = TransactionRequest::default()
        .with_to(target_address)
        .with_nonce(0)
        .with_chain_id(CHAIN_ID)
        .with_input(envelope.data)
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
) -> Result<alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>> {
    let signer: PrivateKeySigner = private_key.unwrap().parse()?;
    let wallet = EthereumWallet::from(signer.clone());
    let nonce = provider
        .get_transaction_count(signer.clone().address())
        .await?;

    let tx = TransactionRequest::default()
        .with_to(Address::ZERO)
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
    envelope_inputs: Vec<Envelope>,
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

    let bundle = BundleData::from(envelopes.clone());
    let serialized = TxEnvelopeWrapper::borsh_ser(&bundle);
    let compressed = TxEnvelopeWrapper::brotli_compress(&serialized);

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

pub async fn retrieve_bundle_tx(txid: String) -> Result<GetBlockFromTx> {
    let provider = create_evm_http_client(WVM_RPC_URL).await?;
    let txid = B256::from_str(&txid)?;
    let tx = provider.get_transaction_by_hash(txid).await?;
    let tx_json = serde_json::json!(&tx.unwrap());

    let block_hash: &str = tx_json["blockHash"].as_str().unwrap_or("0x");
    let block_number_hex: &str = tx_json["blockNumber"].as_str().unwrap_or("0x");
    let block_number_dec = U256::from_str(block_number_hex).unwrap_or(U256::ZERO);
    let calldata: &str = tx_json["input"].as_str().unwrap_or("0x");

    let res = GetBlockFromTx::from(
        block_number_dec.to_string(),
        block_hash.to_string(),
        calldata.to_string(),
    );
    Ok(res)
}

pub async fn retrieve_bundle_data(calldata: String) -> BundleData {
    let byte_array = hex::decode(calldata.trim_start_matches("0x")).expect("decoding failed");
    let unbrotli = TxEnvelopeWrapper::brotli_decompress(byte_array);
    let unborsh: BundleData = TxEnvelopeWrapper::borsh_der(unbrotli);
    unborsh
}
