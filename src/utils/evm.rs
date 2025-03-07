use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::bundle_tx_metadata::BundleTxMetadata;
use crate::utils::core::envelope::Envelope;
use crate::utils::core::tx_envelope_writer::TxEnvelopeWrapper;
use crate::utils::errors::Error;
use alloy::signers::Signer;
use {
    crate::utils::constants::{ADDRESS_BABE1, BLOCK_GAS_LIMIT, CHAIN_ID, WVM_RPC_URL},
    alloy::{
        consensus::TxEnvelope,
        network::{EthereumWallet, TransactionBuilder},
        primitives::{Address, B256, U256},
        providers::{Provider, ProviderBuilder, RootProvider},
        rpc::types::{Transaction, TransactionRequest},
        signers::local::{LocalSigner, PrivateKeySigner},
        transports::http::{Client, Http},
    },
    eyre::OptionExt,
    futures::future::join_all,
    hex,
    rand::{thread_rng, Rng, RngCore},
    serde_json,
    std::str::FromStr,
    tokio::task,
};

pub type HttpClient = RootProvider<Http<Client>>;
pub type AlloyPk = alloy::primitives::FixedBytes<32>;

pub async fn create_evm_http_client(rpc_url: &str) -> Result<HttpClient, Error> {
    let rpc_url = rpc_url.parse().map_err(|_| Error::InvalidRpcUrl)?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    Ok(provider)
}

pub async fn create_envelope(
    private_key: Option<&str>,
    envelope: Envelope,
) -> Result<TxEnvelope, Error> {
    if let Some(priv_key) = private_key {
        let signer: PrivateKeySigner = priv_key
            .parse()
            .map_err(|_| Error::PrivateKeyParsingError)?;
        let wallet = EthereumWallet::from(signer.clone());
        let envelope_target_address = envelope
            .target
            .map(|t| t.parse::<Address>().unwrap_or(Address::ZERO))
            .unwrap_or(Address::ZERO);

        let envelope_data = envelope
            .data
            .ok_or_else(|| Error::Other("Data Required".to_string()))?;

        let tx = TransactionRequest::default()
            .with_to(envelope_target_address)
            .with_nonce(0)
            .with_chain_id(CHAIN_ID)
            .with_input(envelope_data)
            .with_value(U256::from(0))
            .with_gas_limit(0)
            .with_gas_price(0);

        let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;
        Ok(tx_envelope)
    } else {
        Err(Error::PrivateKeyNeeded)
    }
}

// async fn broadcast_bundle(
//     envelopes: Vec<u8>,
//     provider: &RootProvider<Http<Client>>,
//     private_key: Option<String>,
//     version: &str,
// ) -> Result<
//     alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>,
//     Error,
// > {
//     if let Some(priv_key) = private_key {
//         let signer: PrivateKeySigner = priv_key.parse()?;
//         let wallet = EthereumWallet::from(signer.clone());

//         let mut nonce = provider
//             .get_transaction_count(signer.clone().address())
//             .await?;
//         let mut max_priority_fee_per_gas: u128 = 1_000_000_000;
//         let mut max_fee_per_gas: u128 = 2_000_000_000;

//         println!("Initial nonce: {:?}", nonce);

//         let mut attempt = 0;

//         loop {
//             attempt += 1;
//             println!("Broadcast attempt: {}", attempt);

//             let tx = TransactionRequest::default()
//                 .with_to(version.parse::<Address>()?)
//                 .with_nonce(nonce)
//                 .with_chain_id(CHAIN_ID)
//                 .with_input(envelopes.clone())
//                 .with_value(U256::from(0))
//                 .with_gas_limit(490_000_000)
//                 .with_max_priority_fee_per_gas(max_priority_fee_per_gas)
//                 .with_max_fee_per_gas(max_fee_per_gas);

//             let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;

//             match provider.send_tx_envelope(tx_envelope.clone()).await {
//                 Ok(tx) => {
//                     println!("Transaction successfully broadcasted with nonce: {}", nonce);
//                     return Ok(tx);
//                 }
//                 Err(e)
//                     if e.to_string()
//                         .contains("replacement transaction underpriced") =>
//                 {
//                     println!("Transaction underpriced, trying next nonce...");
//                     nonce += 1; // increment nonce if underpriced

//                     if max_fee_per_gas < BLOCK_GAS_LIMIT
//                         && max_priority_fee_per_gas < BLOCK_GAS_LIMIT
//                     {
//                         max_priority_fee_per_gas *= 11 / 10; // 1.1 -> 10% increment
//                         max_fee_per_gas *= 11 / 10; // same
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Unexpected error: {:?}", e);
//                     return Err(e.into());
//                 }
//             }
//         }
//     } else {
//         Err(Error::PrivateKeyNeeded)
//     }
// }

async fn broadcast_bundle(
    envelopes: Vec<u8>,
    provider: &RootProvider<Http<Client>>,
    private_key: Option<String>,
    version: &str,
) -> Result<
    alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>,
    Error,
> {
    if let Some(priv_key) = private_key {
        let signer: PrivateKeySigner = priv_key.parse()?;
        let wallet = EthereumWallet::from(signer.clone());
        let address = signer.address();

        let mut nonce = provider.get_transaction_count(address).await?;
        let version_address = version.parse::<Address>()?;
        let mut max_priority_fee_per_gas: u128 = 1_000_000_000;
        let mut max_fee_per_gas: u128 = 2_000_000_000;

        let mut attempt = 0;

        let envelopes_ref = &envelopes;

        loop {
            attempt += 1;
            // println!("Broadcast attempt: {}", attempt);

            let tx = TransactionRequest::default()
                .with_to(version_address)
                .with_nonce(nonce)
                .with_chain_id(CHAIN_ID)
                .with_input(envelopes_ref.clone())
                .with_value(U256::from(0))
                .with_gas_limit(490_000_000)
                .with_max_priority_fee_per_gas(max_priority_fee_per_gas)
                .with_max_fee_per_gas(max_fee_per_gas);

            let tx_envelope: alloy::consensus::TxEnvelope = tx.build(&wallet).await?;

            match provider.send_tx_envelope(tx_envelope).await {
                Ok(tx) => {
                    println!("Transaction successfully broadcasted with nonce: {}", nonce);
                    return Ok(tx);
                }
                Err(e)
                    if e.to_string()
                        .contains("replacement transaction underpriced") =>
                {
                    println!("Transaction underpriced, trying next nonce...");
                    nonce += 1; // increment nonce if underpriced

                    if max_fee_per_gas < BLOCK_GAS_LIMIT
                        && max_priority_fee_per_gas < BLOCK_GAS_LIMIT
                    {
                        max_priority_fee_per_gas *= 11 / 10;
                        max_fee_per_gas *= 11 / 10;
                    }
                }
                Err(e) => {
                    eprintln!("Unexpected error: {:?}", e);
                    return Err(e.into());
                }
            }
        }
    } else {
        Err(Error::PrivateKeyNeeded)
    }
}

pub async fn create_bundle(
    mut provider: Option<HttpClient>,
    envelope_inputs: Vec<Envelope>,
    private_key: String,
    version: &str,
) -> Result<
    alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>,
    Error,
> {
    if provider.is_none() {
        println!("PROVIDER NOT PROVIDED");
        provider = Some(create_evm_http_client(WVM_RPC_URL).await?);
    }
    let provider = std::sync::Arc::new(provider.unwrap());
    let private_key = private_key.clone();

    let futures: Vec<_> = envelope_inputs
        .into_iter()
        .enumerate()
        .map(|(i, input)| {
            let pk = private_key.clone();
            task::spawn(async move {
                match create_envelope(Some(&pk), input.clone()).await {
                    Ok(tx) => {
                        println!("created tx count {}", i);
                        Ok(TxEnvelopeWrapper::from_envelope(tx, input))
                    }
                    Err(e) => Err(e),
                }
            })
        })
        .collect();

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
        broadcast_bundle(compressed, &provider, Some(private_key), version).await?;

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

pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut data = vec![0u8; length];
    thread_rng().fill_bytes(&mut data);
    data
}

pub async fn retrieve_bundle_tx(txid: String) -> Result<BundleTxMetadata, Error> {
    let provider = create_evm_http_client(WVM_RPC_URL).await?;
    let txid = B256::from_str(&txid)?;
    let tx = provider
        .get_transaction_by_hash(txid)
        .await?
        .ok_or_eyre("error retrieving tx");
    let tx_json = serde_json::json!(&tx?);

    let block_hash: &str = tx_json["blockHash"].as_str().unwrap_or("0x");
    let block_number_hex: &str = tx_json["blockNumber"].as_str().unwrap_or("0x");
    let block_number_dec = U256::from_str(block_number_hex).unwrap_or(U256::ZERO);
    let calldata: &str = tx_json["input"].as_str().unwrap_or("0x");
    let to: &str = tx_json["to"]
        .as_str()
        .unwrap_or("0x0000000000000000000000000000000000000000");

    let res = BundleTxMetadata::from(
        block_number_dec.to_string(),
        block_hash.to_string(),
        calldata.to_string(),
        to.to_string(),
    );
    Ok(res)
}

pub async fn retrieve_bundle_data(calldata: String) -> BundleData {
    let byte_array = hex::decode(calldata.trim_start_matches("0x")).expect("decoding failed");
    let unbrotli = TxEnvelopeWrapper::brotli_decompress(byte_array);
    let unborsh: BundleData = TxEnvelopeWrapper::borsh_der(unbrotli);
    // validate envelopes MUSTs
    for envelope in &unborsh.envelopes {
        assert_eq!(envelope.nonce, 0);
        assert_eq!(envelope.gas_limit, 0);
        assert_eq!(envelope.gas_price, 0);
    }

    unborsh
}

pub async fn sign_data(private_key: Option<&str>, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    if let Some(priv_key) = private_key {
        let signer: PrivateKeySigner = priv_key
            .parse()
            .map_err(|_| Error::PrivateKeyParsingError)?;

        let signed_msg = signer
            .sign_message(&data)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
        // [u8; 65] to Vec<u8>
        Ok(signed_msg.as_bytes().to_vec())
    } else {
        Err(Error::PrivateKeyNeeded)
    }
}

pub async fn send_wvm(
    sender_pk: AlloyPk,
    address_to: Address,
    amount: u64,
) -> Result<String, Error> {
    let gas_price = U256::from(1_200_000_000);
    let amount_wei = U256::from(amount);
    let signer =
        LocalSigner::from_bytes(&sender_pk).map_err(|err| Error::Other(err.to_string()))?;
    let wallet = EthereumWallet::from(signer.clone());
    let rpc_url = WVM_RPC_URL.parse().map_err(|_| Error::InvalidRpcUrl)?;
    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url);
    let nonce = provider.get_transaction_count(signer.address()).await?;

    let tx = TransactionRequest::default()
        .with_to(address_to)
        .with_value(amount_wei)
        .with_gas_price(gas_price.try_into().unwrap())
        .with_gas_limit(500_000)
        .with_nonce(nonce)
        .gas_limit(BLOCK_GAS_LIMIT.try_into().unwrap());

    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|err| Error::Other(err.to_string()))?;

    Ok(receipt.transaction_hash.to_string())
}

pub async fn create_bundle_sync(
    mut provider: Option<HttpClient>,
    envelope_inputs: Vec<Envelope>,
    private_key: String,
    version: &str,
) -> Result<
    alloy::providers::PendingTransactionBuilder<Http<Client>, alloy::network::Ethereum>,
    Error,
> {
    if provider.is_none() {
        provider = Some(create_evm_http_client(WVM_RPC_URL).await?);
    }
    let provider = std::sync::Arc::new(provider.unwrap());
    let mut envelopes = Vec::with_capacity(envelope_inputs.len());

    for (i, input) in envelope_inputs.into_iter().enumerate() {
        match create_envelope(Some(&private_key), input.clone()).await {
            Ok(tx) => {
                // println!("Created envelope {} in {:?}", i, elapsed);
                envelopes.push(TxEnvelopeWrapper::from_envelope(tx, input));
            }
            Err(e) => {
                println!("Failed to create envelope {}: {}", i, e);
                return Err(e);
            }
        }
    }

    let bundle = BundleData::from(envelopes);
    let serialized = TxEnvelopeWrapper::borsh_ser(&bundle);
    let compressed = TxEnvelopeWrapper::brotli_compress_fast(&serialized);

    const MAX_BROADCAST_RETRIES: usize = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_BROADCAST_RETRIES {
        println!("Broadcast attempt: {}", attempt);
        match broadcast_bundle(
            compressed.clone(),
            &provider,
            Some(private_key.clone()),
            version,
        )
        .await
        {
            Ok(tx) => {
                return Ok(tx);
            }
            Err(e) if attempt < MAX_BROADCAST_RETRIES => {
                println!("Broadcast attempt {} failed: {}", attempt, e);
                last_error = Some(e);
            }
            Err(e) => {
                println!("Final broadcast attempt failed: {}", e);
                last_error = Some(e);
                break;
            }
        }
    }

    Err(last_error.unwrap_or(Error::Other("All broadcast attempts failed".to_string())))
}
