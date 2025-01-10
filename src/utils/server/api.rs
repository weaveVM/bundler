use crate::utils::core::bundle_data::BundleData;
use crate::utils::core::{bundle::Bundle, envelope};
use axum::{extract::Path, response::Json};
use serde_json::Value;

pub async fn get_greet() -> &'static str {
    "running UwU"
}

pub async fn get_envelopes_of(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id).await.unwrap();
    Json(serde_json::to_value(&envelopes).unwrap())
}

pub async fn get_envelopes_id_of(Path(id): Path<String>) -> Json<Value> {
    let envelopes: BundleData = Bundle::retrieve_envelopes(id).await.unwrap();
    let envelopes_ids: Vec<String> = envelopes.envelopes.into_iter().map(|tx| tx.hash).collect();
    Json(serde_json::to_value(&envelopes_ids).unwrap())
}
