use crate::utils::server::api::{
    get_envelopes_id_of, get_envelopes_id_of_2, get_envelopes_of, get_envelopes_of_2,
    get_envelopes_of_full, get_envelopes_of_full_2, get_greet, resolve_large_bundle,
};
use axum::{routing::get, Router};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use tokio::net::TcpListener;

pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(3600));
    
    // server routes
    let app = Router::new()
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

    // Get port from environment variable or default to 3000
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("Server running on http://{}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
