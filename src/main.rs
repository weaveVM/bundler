use crate::utils::server::api::{
    get_envelopes_id_of, get_envelopes_id_of_2, get_envelopes_of, get_envelopes_of_2,
    get_envelopes_of_full, get_envelopes_of_full_2, get_greet, resolve_large_bundle,
};
use axum::{routing::get, Router};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

pub mod utils;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(3600));
    // server routes
    let router = Router::new()
        .route("/", get(get_greet))
        // v1 routes
        .route("/v1/envelopes/{bundle_txid}", get(get_envelopes_of))
        .route("/v1/envelopes/ids/{bundle_txid}", get(get_envelopes_id_of))
        .route(
            "/v1/envelopes-full/{bundle_txid}",
            get(get_envelopes_of_full),
        )
        // v2 routes
        .route("/v2/envelopes/{bundle_txid}", get(get_envelopes_of_2))
        .route("/v2/envelopes/ids/{bundle_txid}", get(get_envelopes_id_of_2))
        .route(
            "/v2/envelopes-full/{bundle_txid}",
            get(get_envelopes_of_full_2),
        )
        .route("/v2/resolve/{large_bundle_txid}", get(resolve_large_bundle))
        .layer(timeout_layer);

    Ok(router.into())
}
