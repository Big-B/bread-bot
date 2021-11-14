use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Arc;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::UserId, prelude::ReactionType},
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

struct Action {
    regex: Option<Arc<Regex>>,
    reaction: ReactionType,
}

struct Handler {
    map: HashMap<UserId, Vec<Action>>,
    list: Vec<(Regex, ReactionType)>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Make sure user is defined
        if let Some(actions) = self.map.get(&msg.author.id) {
            // Check all actions for this user
            for action in actions {
                // Check for regex
                if let Some(re) = &action.regex {
                    // Only act if the regex matches
                    if re.is_match(&msg.content) {
                        if let Err(why) = msg.react(ctx.http.clone(), action.reaction.clone()).await
                        {
                            println!("Error reacting to message: {:?}", why);
                        }
                    }
                } else {
                    // No regex so just react
                    if let Err(why) = msg.react(ctx.http.clone(), action.reaction.clone()).await {
                        println!("Error reacting to message: {:?}", why);
                    }
                }
            }
        }

        // Run through the list of actions that are user independent
        for (_, e) in self.list.iter().filter(|(r, _)| r.is_match(&msg.content)) {
            if let Err(why) = msg.react(ctx.http.clone(), e.clone()).await {
                println!("Error reacting to message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
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
    let mut map = HashMap::new();
    let mut list = Vec::new();
    for action in config_data.actions {
        match (&action.users, &action.filter) {
            (Some(users), _) => {
                // Check to see if a regex was provided and if it's a valid regex
                let r: Option<Arc<Regex>> = action
                    .filter
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

    let mut client = Client::builder(&config_data.discord_token)
        .event_handler(Handler { map, list })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
