use crate::utils::constants::{ADDRESS_BABE1, ADDRESS_BABE2};
use crate::utils::core::bundle::Bundle;
use crate::utils::core::bundle_data::BundleData;
use axum::{extract::Path, response::Json};
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
