use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub postgres_url: String,
    pub application_id: u64,
}
