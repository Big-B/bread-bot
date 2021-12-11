mod action;
mod handler;
mod schema;
mod reaction_set;
#[macro_use]
extern crate diesel;
use crate::handler::Handler;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::Deserialize;
use serenity::prelude::*;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Deserialize)]
struct Config {
    discord_token: String,
    postgres_url: String,
    application_id: u64,
}

#[tokio::main]
async fn main() {
    // Read in config file
    let mut reader = BufReader::new(
        File::open("/etc/bread-bot.toml")
            .expect("Expected /etc/bread-bot.toml to exist and be readable"),
    );

    // Parse config file
    let mut config_data = String::new();
    reader
        .read_to_string(&mut config_data)
        .expect("Expected valid UTF-8 in config file");

    let config_data: Config = toml::from_str(&config_data).expect("Invalid config file format");
    let connection = PgConnection::establish(&config_data.postgres_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", config_data.postgres_url));

    // Loop over configured action and convert them to a HashMap
    let mut client = Client::builder(&config_data.discord_token)
        .application_id(config_data.application_id)
        .event_handler(Handler::new(Arc::new(Mutex::new(connection))))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
