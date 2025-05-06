use crate::utils::constants::LOAD0_ENDPOINT_URL;
use crate::utils::errors::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Load0UploadResponse {
    pub optimistic_hash: String,
    pub success: bool,
}

pub async fn upload_to_load0(data: Vec<u8>, content_type: Option<String>, api_key: Option<String>) -> Result<String, Error> {
    let client = Client::new();
    let api_key = api_key.unwrap_or_default();
    let upload_url = format!("{}/upload", LOAD0_ENDPOINT_URL);

    let response = client
        .post(&upload_url)
        .header("Content-Type", content_type.unwrap_or("octet-stream".to_string()))
        .header("X-Load-Authorization", api_key)
        .body(data)
        .send()
        .await.map_err(|err| Error::Other(err.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::Other("Error sending data to load0".to_string()));
    }

    let upload_response = response.json::<Load0UploadResponse>().await.map_err(|err| Error::Other(err.to_string()))?;

    if (upload_response.success) {
        return Ok(upload_response.optimistic_hash);
    }

    Ok(String::from(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    ))
}