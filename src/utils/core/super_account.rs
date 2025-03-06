use crate::utils::constants::{SAFE_CHUNK_TOPUP, WVM_RPC_URL};
use crate::utils::errors::Error;
use crate::utils::evm::{create_evm_http_client, send_wvm, AlloyPk, HttpClient};
use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::signers::{k256, local::LocalSigner, local::PrivateKeySigner, Signer};
use eyre::OptionExt;
use futures::future::join_all;
use futures::stream::All;
use futures::{stream, StreamExt};
use rand::{thread_rng, RngCore};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time::Instant;

use alloy::signers::k256::Secp256k1;
use ecdsa::SigningKey;
use std::str::FromStr;

pub type Chunker = LocalSigner<SigningKey<Secp256k1>>;

#[derive(Debug, Clone, Default)]
pub struct SuperAccount {
    pub funder: Option<AlloyPk>,
    pub keystore_path: Option<String>,
    pub keystore_pwd: Option<String>,
    pub chunkers: Option<Vec<Chunker>>,
}

impl SuperAccount {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn funder(mut self, key: String) -> Self {
        let funder_pk: AlloyPk = B256::from_str(key.trim_start_matches("0x")).unwrap();
        self.funder = Some(funder_pk);

        self
    }

    pub fn keystore_path(mut self, path: String) -> Self {
        self.keystore_path = Some(path);
        self
    }

    pub fn pwd(mut self, pwd: String) -> Self {
        self.keystore_pwd = Some(pwd);
        self
    }

    pub async fn fund_chunkers(mut self) -> Result<Self, Error> {
        let chunkers = self
            .chunkers
            .clone()
            .ok_or_eyre("Error: chunkers not found")?;
        let funder = self.funder.ok_or_eyre("Error: funder not provided")?;
        let http_client = create_evm_http_client(WVM_RPC_URL)
            .await
            .map_err(|err| Error::Other(err.to_string()))?;

        for chunker in chunkers {
            println!("FUNDING {}", chunker.address());
            send_wvm(funder, chunker.address(), SAFE_CHUNK_TOPUP).await?;
        }

        Ok(self)
    }

    pub async fn create_chunkers(self, amount: u32) -> Result<Self, Error> {
        let path = self
            .clone()
            .keystore_path
            .ok_or_eyre("Error: keystore path not provided")?;
        let pwd = self
            .clone()
            .keystore_pwd
            .ok_or_eyre("Error: keystore pwd not provided")?;

        let keystore_dir = Path::new(&path);
        std::fs::create_dir_all(keystore_dir).map_err(|err| Error::Other(err.to_string()))?;

        let wallets = Arc::new(Mutex::new(HashMap::new()));
        let keystore_dir = Arc::new(keystore_dir.to_path_buf());

        let mut handles = Vec::with_capacity(amount as usize);

        for i in 0..amount {
            let wallets = Arc::clone(&wallets);
            let keystore_dir = Arc::clone(&keystore_dir);
            let password = pwd.to_string();

            let handle = task::spawn(async move {
                let task_start = Instant::now();

                let mut rng = thread_rng();
                let mut bytes = [0u8; 32];
                rng.fill_bytes(&mut bytes);
                let private_key: alloy::primitives::FixedBytes<32> = B256::from(bytes);

                // encrypt to keystore
                if let Ok((signer, keystore_path)) = LocalSigner::encrypt_keystore(
                    keystore_dir.as_ref(),
                    &mut rng,
                    private_key,
                    &password,
                    Some(format!("wallet_{}.json", i).as_str()),
                ) {
                    let address: Address = signer.address();
                    let full_path = keystore_dir.join(&keystore_path);

                    {
                        let mut wallets_lock = wallets.lock().unwrap();
                        wallets_lock.insert(address, full_path);
                    }
                }
            });

            handles.push(handle);
        }
        join_all(handles).await;

        for i in 0..5 {
            let keystore_path = keystore_dir.join(format!("wallet_{}.json", i));

            if keystore_path.exists() {
                let recovered_signer = LocalSigner::decrypt_keystore(&keystore_path, &pwd)?;
                println!(
                    "Recovered wallet_{}: {} from {:?}",
                    i,
                    recovered_signer.address(),
                    keystore_path
                );
            } else {
                println!(
                    "Keystore file for wallet_{} not found at {:?}",
                    i, keystore_path
                );
            }
        }

        Ok(self)
    }

    pub async fn load_chunkers(mut self, count: Option<u32>) -> Result<Self, Error> {
        let mut chunkers: Vec<Chunker> = Vec::new();
        let path = self
            .clone()
            .keystore_path
            .ok_or_eyre("Error: keystore path not provided")?;
        let pwd = self
            .clone()
            .keystore_pwd
            .ok_or_eyre("Error: keystore pwd not provided")?;

        let keystore_dir = Path::new(&path);
        let count = fs::read_dir(keystore_dir)
            .map_err(|err| Error::Other(err.to_string()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "json")
            })
            .count();

        for i in 0..count {
            let keystore_path = keystore_dir.join(format!("wallet_{}.json", i));

            if keystore_path.exists() {
                let recovered_signer = LocalSigner::decrypt_keystore(&keystore_path, &pwd)?;
                println!(
                    "Recovered wallet_{}: {} from {:?}",
                    i,
                    recovered_signer.address(),
                    keystore_path
                );
                chunkers.push(recovered_signer);
            } else {
                println!(
                    "Keystore file for wallet_{} not found at {:?}",
                    i, keystore_path
                );
            }
        }

        self.chunkers = Some(chunkers);

        Ok(self)
    }
}
