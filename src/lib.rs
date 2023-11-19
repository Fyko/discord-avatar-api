// axum::debug_handler is angry without allowing diverging_sub_expression
#![allow(clippy::diverging_sub_expression)]

#[macro_use]
extern crate serde;

use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::MatchedPath,
    http::{HeaderName, Request},
    middleware,
    routing::{any, get},
    BoxError, Error, Router,
};
use http::StatusCode;
use metrics::track_metrics;
use request_id::MyRequestId;
use routes::handle_query;
use state::AppState;
use std::time::Duration;
use tower::{
    timeout::{error::Elapsed, TimeoutLayer},
    ServiceBuilder,
};
use tower_governor::{
    errors::display_error, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info_span;

pub type Result<T, E = Error> = anyhow::Result<T, E>;

pub mod config;
pub mod fetch_user;
pub mod metrics;
pub mod request_id;
pub mod routes;
pub mod state;

pub fn create_router(state: AppState) -> Router<()> {
    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            if e.is::<Elapsed>() {
                (StatusCode::REQUEST_TIMEOUT, e.to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let x_request_id = HeaderName::from_static("x-request-id");

    let request_id_layer = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MyRequestId::default(),
        ))
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str);

        info_span!(
          "http_request",
          method = ?request.method(),
          matched_path,
        )
    });

    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let governor_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            display_error(e)
        }))
        .layer(GovernorLayer {
            config: Box::leak(governor_conf),
        });

    let compression_layer = CompressionLayer::new().gzip(true).br(true).zstd(true);

    let middleware = ServiceBuilder::new()
        .layer(timeout_layer)
        .layer(request_id_layer)
        .layer(trace_layer)
        .layer(governor_layer)
        .layer(compression_layer)
        .layer(middleware::from_fn(track_metrics));

    Router::new().merge(routes(state)).layer(middleware)
}

fn routes(state: AppState) -> Router<()> {
    Router::new()
        .merge(metrics::routes())
        .route("/health", get(debug))
        .route("/*path", get(handle_query))
        .route("/", any(debug))
        .with_state(state)
}

#[axum::debug_handler]
async fn debug() -> &'static str {
    "Hello, World!"
}
