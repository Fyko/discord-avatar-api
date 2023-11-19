use std::{str::FromStr, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use twilight_http::Client;
use twilight_model::{
    id::{marker::UserMarker, Id},
    user::User,
};

pub const ALLOWED_SIZES: &[i16] = &[16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// Fetches a user from the Dicsord API.
pub async fn fetch_user(client: Arc<Client>, id: Id<UserMarker>) -> Result<User, Response> {
    let res = client.user(id).await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch user from Discord",
        )
            .into_response()
    })?;

    let user = res.model().await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to deserialize user from Discord",
        )
            .into_response()
    })?;

    Ok(user)
}

/// The formats that a user's avatar can be returned in.
pub enum UserAvatarFormats {
    Png,
    Jpg,
    Webp,
    Gif,
}

impl FromStr for UserAvatarFormats {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(UserAvatarFormats::Png),
            "jpg" | "jpeg" => Ok(UserAvatarFormats::Jpg),
            "webp" => Ok(UserAvatarFormats::Webp),
            "gif" => Ok(UserAvatarFormats::Gif),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for UserAvatarFormats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserAvatarFormats::Png => f.write_str("png"),
            UserAvatarFormats::Jpg => f.write_str("jpg"),
            UserAvatarFormats::Webp => f.write_str("webp"),
            UserAvatarFormats::Gif => f.write_str("gif"),
        }
    }
}

/// Resolves a user's avatar URL.
pub fn resolve_avatar(user: User, format: Option<UserAvatarFormats>, size: i16) -> String {
    let avatar_hash = match user.avatar {
        Some(avatar) => avatar,
        None => return resolve_default_avatar(user),
    };
    let animated = avatar_hash.is_animated();

    let format = format.unwrap_or(if animated {
        UserAvatarFormats::Gif
    } else {
        UserAvatarFormats::Png
    });

    let id = user.id.get();

    format!("https://cdn.discordapp.com/avatars/{id}/{avatar_hash}.{format}?size={size}")
}

/// Calculates the default avatar index for a user.
#[inline]
pub fn calculate_default_avatar_index(user: User) -> u8 {
    if user.discriminator == 0 {
        ((user.id.get() >> 22) % 6) as u8
    } else {
        (user.discriminator % 5) as u8
    }
}

/// Resolves a user's default avatar URL.
#[inline]
fn resolve_default_avatar(user: User) -> String {
    let index = calculate_default_avatar_index(user);

    format!("https://cdn.discordapp.com/embed/avatars/{index}.png")
}
