use crate::utils::server::api::{get_envelopes_id_of, get_envelopes_of, get_greet};
use axum::{routing::get, Router};

pub mod utils;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // server routes
    let router = Router::new()
        .route("/", get(get_greet))
        .route("/v1/envelopes/:bundle_txid", get(get_envelopes_of))
        .route("/v1/envelopes/ids/:bundle_txid", get(get_envelopes_id_of));

    Ok(router.into())
}
