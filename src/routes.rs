use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use http::{header, StatusCode};
use once_cell::sync::Lazy;
use regex::Regex;
use twilight_model::id::{marker::UserMarker, Id};

use crate::{
    fetch_user::{fetch_user, resolve_avatar, UserAvatarFormats, ALLOWED_SIZES},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    size: Option<i16>,
}

static REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?<id>\d{17,19})(\.(?<format>png|webp|jpeg|gif))?$").unwrap());

#[axum::debug_handler]
pub async fn handle_query(
    State(state): State<AppState>,
    Path(full_path): Path<String>,
    Query(query): Query<QueryParams>,
) -> Response {
    tracing::info!("Handling request for {} with query {:?}", full_path, query);

    let size = query.size.unwrap_or(512);
    if !ALLOWED_SIZES.contains(&size) {
        return (StatusCode::BAD_REQUEST, "Invalid size").into_response();
    }

    let caps = match REGEX.captures(&full_path) {
        Some(caps) => caps,
        None => return (StatusCode::BAD_REQUEST, "Invalid path").into_response(),
    };

    let id = caps.name("id").map_or("", |m| m.as_str());
    if id.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid id").into_response();
    }

    let user_id = match Id::<UserMarker>::from_str(id) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid user ID").into_response(),
    };
    let user = match fetch_user(state.discord_client.clone(), user_id).await {
        Ok(user) => user,
        Err(res) => return res,
    };

    let format = caps.name("format");
    let format =
        format.map(|format| UserAvatarFormats::from_str(format.as_str()).expect("infalible"));

    let avatar = resolve_avatar(user, format, size);

    (
        StatusCode::FOUND,
        [
            (header::LOCATION, avatar),
            (header::CACHE_CONTROL, "max-age=21600".to_string()),
        ],
    )
        .into_response()
}
