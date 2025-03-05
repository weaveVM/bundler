use crate::utils::constants::{ADDRESS_BABE1, ADDRESS_BABE2};
use crate::utils::core::bundle::Bundle;
use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::large_bundle::LargeBundle;
use axum::response::IntoResponse;
use axum::{extract::Path, response::Json};
use reqwest::{header, StatusCode};
use serde_json::Value;

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
    let large_bundle = LargeBundle::retrieve_chunks_receipts(id).await.unwrap();
    let lb_data = large_bundle
        .clone()
        .reconstruct_large_bundle()
        .await
        .unwrap();

    (
        [
            (
                header::CONTENT_TYPE,
                large_bundle.content_type.unwrap_or_default(),
            ),
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000".to_string(),
            ),
        ],
        lb_data,
    )
        .into_response()
}
