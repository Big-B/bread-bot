extern crate diesel;
use crate::action::Action;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::sync::Arc;
use std::sync::Mutex;

pub struct Handler {
    db_con: Arc<Mutex<PgConnection>>,
}

impl Handler {
    pub fn new(db_con: Arc<Mutex<PgConnection>>) -> Self {
        Self { db_con }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::actions::dsl::*;

        // Grab the results of the query and minimize the scope of the db lock
        // Looking for either a matching or null author in the proper guild
        let results = {
            let db = self.db_con.lock().unwrap();
            actions
                .filter(guild_id.eq(msg.guild_id.expect("No guild ID for message").0 as i64))
                .filter(user_id.eq(msg.author.id.0 as i64).or(user_id.is_null()))
                .load::<Action>(&*db)
                .expect("Query Failed")
        };

        // Gather all the reactions. If there is a regex, check it, if not then
        // just add the reaction
        let mut all_reactions = String::new();
        for action in results {
            if let Some(s) = action.regex {
                // There is a regex, so see if it matches
                let r = Regex::new(&s).unwrap();
                if r.is_match(&msg.content) {
                    for reaction in action.reactions {
                        all_reactions.push_str(&reaction);
                    }
                }
            } else {
                // No regex, so just add to list
                for reaction in action.reactions {
                    all_reactions.push_str(&reaction);
                }
            }
        }

        // Go through all the reactions and react to the message appropriately
        for reaction in all_reactions.chars() {
            if let Err(why) = msg.react(ctx.http.clone(), reaction).await {
                println!("Error reacting to message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
