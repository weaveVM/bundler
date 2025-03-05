use crate::utils::constants::{ADDRESS_BABE1, ADDRESS_BABE2};
use crate::utils::core::bundle::Bundle;
use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::large_bundle::LargeBundle;
use crate::utils::errors::Error;
use axum::body::Body;
use axum::response::IntoResponse;
use axum::{extract::Path, response::Json};
use bytes::Bytes;
use futures::stream::{self};
use reqwest::{header, StatusCode};
use serde_json::Value;
use std::convert::Infallible;

pub async fn get_greet() -> &'static str {
    "running UwU"
}

pub async fn get_envelopes_of(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE1).await.unwrap();
    Json(serde_json::to_value(&envelopes).unwrap())
}

pub async fn get_envelopes_of_full(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE1).await.unwrap();
    let envelopes_with_owners = BundleData::to_bundle_with_owners(envelopes).await;
    Json(serde_json::to_value(&envelopes_with_owners).unwrap())
}

pub async fn get_envelopes_id_of(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE2).await.unwrap();
    let envelopes_ids: Vec<String> = envelopes.envelopes.into_iter().map(|tx| tx.hash).collect();
    Json(serde_json::to_value(&envelopes_ids).unwrap())
}

pub async fn get_envelopes_of_2(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE2).await.unwrap();
    Json(serde_json::to_value(&envelopes).unwrap())
}

pub async fn get_envelopes_of_full_2(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE2).await.unwrap();
    let envelopes_with_owners = BundleData::to_bundle_with_owners(envelopes).await;
    Json(serde_json::to_value(&envelopes_with_owners).unwrap())
}

pub async fn get_envelopes_id_of_2(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id, ADDRESS_BABE2).await.unwrap();
    let envelopes_ids: Vec<String> = envelopes.envelopes.into_iter().map(|tx| tx.hash).collect();
    Json(serde_json::to_value(&envelopes_ids).unwrap())
}

pub async fn resolve_large_bundle(Path(id): Path<String>) -> impl IntoResponse {
    let large_bundle = match LargeBundle::retrieve_chunks_receipts(id.clone()).await {
        Ok(bundle) => bundle,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error retrieving bundle: {}", e),
            )
                .into_response();
        }
    };

    let content_type = large_bundle
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Get the chunk receipts
    let chunks_receipts = match &large_bundle.chunks_receipts {
        Some(receipts) => receipts.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No chunk receipts found".to_string(),
            )
                .into_response();
        }
    };

    if chunks_receipts.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Bundle has no chunks".to_string(),
        )
            .into_response();
    }

    // For MP4 files, we need to ensure we send content-length and avoid chunked encoding
    // We'll calculate the total size by fetching metadata about each chunk first
    let is_mp4 = content_type == "video/mp4" || content_type.starts_with("video/");

    if is_mp4 {
        // For MP4 files, we need to ensure we can use content-length
        // We'll create a stream that loads chunks one by one but doesn't use chunked encoding

        // First, create a streaming body that processes chunks sequentially
        let stream = stream::unfold(
            (0, chunks_receipts.clone()),
            move |(chunk_index, chunks_receipts)| async move {
                if chunk_index >= chunks_receipts.len() {
                    return None;
                }

                // Get the chunk receipt at the current index
                let chunk_receipt = &chunks_receipts[chunk_index];

                // Retrieve this chunk's data
                match retrieve_chunk_data(chunk_receipt).await {
                    Ok(chunk_data) => Some((
                        Ok::<_, Infallible>(Bytes::from(chunk_data)),
                        (chunk_index + 1, chunks_receipts),
                    )),
                    Err(e) => {
                        eprintln!("Error retrieving chunk {}: {}", chunk_index, e);
                        // Skip this chunk and continue
                        Some((Ok(Bytes::from(vec![])), (chunk_index + 1, chunks_receipts)))
                    }
                }
            },
        );

        // Create a streaming body
        let body = Body::from_stream(stream);

        // For MP4 files, it's better to NOT use chunked transfer encoding
        // But we still want to stream immediately, so we'll use a different approach
        // We'll let the client know we accept range requests, which is how browsers stream video
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CACHE_CONTROL, "public, max-age=31536000")
            // Don't set Content-Length for streaming
            .body(body)
            .unwrap()
            .into_response();
    }

    // For non-MP4 files, use normal streaming with chunked encoding
    let stream = stream::unfold(
        (0, chunks_receipts),
        move |(chunk_index, chunks_receipts)| async move {
            if chunk_index >= chunks_receipts.len() {
                return None;
            }

            // Get the chunk receipt at the current index
            let chunk_receipt = &chunks_receipts[chunk_index];

            // Retrieve this chunk's data
            match retrieve_chunk_data(chunk_receipt).await {
                Ok(chunk_data) => Some((
                    Ok::<_, Infallible>(Bytes::from(chunk_data)),
                    (chunk_index + 1, chunks_receipts),
                )),
                Err(e) => {
                    eprintln!("Error retrieving chunk {}: {}", chunk_index, e);
                    Some((Ok(Bytes::from(vec![])), (chunk_index + 1, chunks_receipts)))
                }
            }
        },
    );

    // Create a streaming body
    let body = Body::from_stream(stream);

    // Return response with chunked transfer encoding
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::TRANSFER_ENCODING, "chunked")
        .body(body)
        .unwrap()
        .into_response()
}

// Helper function to retrieve a single chunk's data
async fn retrieve_chunk_data(chunk_receipt: &str) -> Result<Vec<u8>, Error> {
    // Retrieve the bundle for this chunk
    let receipt_bundle = Bundle::retrieve_envelopes(chunk_receipt.to_string(), ADDRESS_BABE2)
        .await
        .map_err(|e| {
            Error::Other(format!(
                "Failed to retrieve bundle for receipt {}: {}",
                chunk_receipt, e
            ))
        })?;

    // Get the envelope data
    let receipt_writer = receipt_bundle
        .envelopes
        .get(0)
        .ok_or_else(|| Error::Other("Error: no envelopes found".to_string()))?;

    // Decode the chunk data
    let receipt_data = hex::decode(&receipt_writer.input.trim_start_matches("0x"))
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(receipt_data)
}
