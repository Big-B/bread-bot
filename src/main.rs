mod handler;
use crate::handler::{Handler, Action};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Arc;

use serenity::{
    model::{id::UserId, prelude::ReactionType},
    prelude::*,
};

#[derive(Deserialize)]
struct ActionInput {
    users: Option<Vec<UserId>>,
    filter: Option<String>,
    reaction: String,
}

#[derive(Deserialize)]
struct Config {
    discord_token: String,
    actions: Vec<ActionInput>,
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

    // Loop over configured action and convert them to a HashMap
    let (map, list) = parse_config_data(&config_data.actions);
    let mut client = Client::builder(&config_data.discord_token)
        .event_handler(Handler { map, list })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn parse_config_data(
    actions: &Vec<ActionInput>,
) -> (HashMap<UserId, Vec<Action>>, Vec<(Regex, ReactionType)>) {
    let mut map = HashMap::new();
    let mut list = Vec::new();

    // Iterate over all the actions parsed
    for action in actions {
        // Validate actions. They must have either a filter, or a user, or both.
        match (&action.users, &action.filter) {
            (Some(users), _) => {
                // Check to see if a regex was provided and if it's a valid regex
                let r: Option<Arc<Regex>> = action
                    .filter
                    .as_ref()
                    .map(|val| Arc::new(Regex::new(&val).expect("Expected valid regex")));

                for user in users {
                    // Insert action into the map
                    map.entry(*user).or_insert_with(Vec::new).push(Action {
                        regex: r.clone(),
                        reaction: ReactionType::Unicode(action.reaction.clone()),
                    });
                }
            }
            (None, Some(filter)) => {
                let r = Regex::new(filter).expect("Expected valid regex");
                list.push((r, ReactionType::Unicode(action.reaction.clone())));
            }
            (None, None) => {
                eprintln!("Entry must have either users or a filter")
            }
        }
    }
    (map, list)
}
