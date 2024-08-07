extern crate diesel;
use crate::action::Action;
use crate::reaction_set::ReactionSet;
use crate::target::{Target, TargetBuilderError};
use diesel::insert_into;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use regex::Regex;
use serenity::{
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::{
        application::{Command, CommandDataOptionValue, CommandOptionType, Interaction},
        channel::Message,
        gateway::Ready,
        id::{GuildId, UserId},
    },
    prelude::*,
};

use serenity::all::ReactionType;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use unicode_segmentation::UnicodeSegmentation;

pub struct Handler {
    db_con: Arc<Mutex<PgConnection>>,
    letter_chain: Arc<Mutex<HashMap<GuildId, (UserId, String)>>>,
}

impl Handler {
    pub fn new(db_con: Arc<Mutex<PgConnection>>) -> Self {
        Self {
            db_con,
            letter_chain: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn target(&self, target: Target) {
        use crate::schema::actions::dsl::*;
        let mut db = self.db_con.lock().unwrap();
        let res = insert_into(actions)
            .values((
                guild_id.eq(target.get_guild().get() as i64),
                user_id.eq(target.get_user().map(|x| x as i64)),
                reactions.eq(target.get_emotes()),
                expiration.eq(target.get_expiration()),
                regex.eq(target.get_regex()),
            ))
            .execute(&mut *db);

        if let Err(e) = res {
            println!("Error inserting target {:?}! {}", target, e);
        }
    }

    fn check_column(&self, msg: &str, gid: GuildId, uid: UserId) -> Option<String> {
        // letter_chain: Arc<Mutex<HashMap<GuildId, (UserId, String)>>>,
        // This is to attempt to handle cases where some loser tries to get around
        // our rules by typing letters out one at a time.
        let mut map = self.letter_chain.lock().unwrap();
        let letters: Vec<&str> = msg.graphemes(true).collect();

        // Pop off the group's entry, if it exists
        let entry = map.remove(&gid);
        // We really only care about single characters
        if letters.len() == 1 {
            // If there's already an entry, and the user matches append to it and return the full
            // string, otherwise start a new entry
            if let Some((user, s)) = entry {
                if user == uid {
                    let s = s + letters[0];
                    map.insert(gid, (uid, s.clone()));
                    return Some(s);
                }
            } else {
                map.insert(gid, (uid, msg.to_string()));
            }
        }
        None
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::actions::dsl::*;

        let time = SystemTime::now();
        let gid = msg.guild_id.expect("No guild ID for message");
        let uid = msg.author.id;

        // Grab the results of the query and minimize the scope of the db lock
        // Looking for either a matching or null author in the proper guild
        let results = {
            let mut db = self.db_con.lock().unwrap();
            actions
                .filter(guild_id.eq(gid.get() as i64))
                .filter(user_id.eq(msg.author.id.get() as i64).or(user_id.is_null()))
                .filter(expiration.is_null().or(expiration.gt(time)))
                .load::<Action>(&mut *db)
                .expect("Query Failed")
        };

        let column = self.check_column(&msg.content, gid, uid);

        // Gather all the reactions. If there is a regex, check it, if not then
        // just add the reaction
        let mut reaction_set = ReactionSet::new();
        for action in results {
            if let Some(s) = action.regex {
                // There is a regex, so see if it matches
                let r = Regex::new(&s).unwrap();
                if r.is_match(&msg.content) {
                    reaction_set.add_reactions(&action.reactions);
                }

                if let Some(s) = &column {
                    if r.is_match(s) {
                        reaction_set.add_reactions(&action.reactions);
                        self.letter_chain.lock().unwrap().remove(&gid);
                    }
                }
            } else {
                // No regex, so just add to list if not conflicting
                reaction_set.add_reactions(&action.reactions);
            }
        }

        // Go through all the reactions and react to the message appropriately
        for reaction in reaction_set.as_list() {
            if let Ok(r) = ReactionType::try_from(reaction.as_str()) {
                if let Err(why) = msg.react(ctx.http.clone(), r).await {
                    println!("Error reacting to message: {:?}", why);
                }
            }
        }

        // Delete any expired rules
        {
            let mut db = self.db_con.lock().unwrap();
            diesel::delete(actions.filter(expiration.lt(time)))
                .execute(&mut *db)
                .expect("Delete failed");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "target_user" | "target_regex" => {
                    let mut builder = Target::builder();
                    builder = builder.set_guild(command.guild_id.unwrap());
                    for entry in &command.data.options {
                        match entry.name.as_ref() {
                            "emotes" => {
                                if let CommandDataOptionValue::String(s) = &entry.value {
                                    builder = builder.set_emotes(s)
                                }
                            }
                            "duration" => {
                                if let CommandDataOptionValue::Integer(int) = &entry.value {
                                    builder = builder.set_expiration(*int as u64)
                                }
                            }
                            "user" => {
                                if let CommandDataOptionValue::User(user) = &entry.value {
                                    builder = builder.set_user(*user)
                                }
                            }
                            "regex" => {
                                if let CommandDataOptionValue::String(s) = &entry.value {
                                    builder = builder.set_regex(s)
                                }
                            }
                            _ => println!("Unexpected entry name: {}", entry.name),
                        }
                    }
                    match builder.build() {
                        Ok(target) => {
                            self.target(target);
                            "Target added".to_string()
                        }
                        Err(TargetBuilderError::BadRegex(_)) => {
                            "Your regex game is weak, bitch. Refer to \
                            https://docs.rs/regex/latest/regex/index.html#syntax"
                                .to_string()
                        }
                        Err(TargetBuilderError::MissingUserAndRegex) => {
                            "Need either a user or a regex or both... bitch".to_string()
                        }
                        Err(e) => e.to_string(),
                    }
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_response(&ctx.http, {
                    let response = CreateInteractionResponseMessage::new().content(content);
                    CreateInteractionResponse::Message(response)
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        // Commands for all servers
        let mut commands = Vec::new();
        let command = CreateCommand::new("target_user")
            .description("Target a user")
            .add_option({
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to target")
                    .required(true)
            })
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "emotes",
                    "List of emotes to target with",
                )
                .required(true)
            })
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "duration",
                    "Length of target in minutes",
                )
                .min_int_value(1)
                .max_int_value(1440)
                .required(true)
            })
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "regex",
                    "Regular expression to match against",
                )
                .required(false)
            });
        commands.push(command);

        let command = CreateCommand::new("target_regex")
            .description("Target message content")
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "regex",
                    "Regular expression to match against",
                )
                .required(true)
            })
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "emotes",
                    "List of emotes to target with",
                )
                .required(true)
            })
            .add_option({
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "duration",
                    "Length of target in minutes",
                )
                .min_int_value(1)
                .max_int_value(1440)
                .required(true)
            })
            .add_option({
                CreateCommandOption::new(CommandOptionType::User, "user", "The user to target")
                    .required(false)
            });
        commands.push(command);

        Command::set_global_commands(&ctx.http, commands)
            .await
            .unwrap();
        println!("{} is connected!", ready.user.name);
    }
}
