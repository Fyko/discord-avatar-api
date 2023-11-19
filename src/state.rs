use std::sync::Arc;
use twilight_http::Client;

#[derive(Clone)]
pub struct InnerAppState {
    pub discord_client: Arc<Client>,
}

pub type AppState = Arc<InnerAppState>;
