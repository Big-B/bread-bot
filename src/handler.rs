extern crate diesel;
use crate::action::Action;
use crate::attack::Attack;
use crate::reaction_set::ReactionSet;
use diesel::insert_into;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

pub struct Handler {
    db_con: Arc<Mutex<PgConnection>>,
}

impl Handler {
    pub fn new(db_con: Arc<Mutex<PgConnection>>) -> Self {
        Self { db_con }
    }

    fn attack(&self, attack: Attack) {
        use crate::schema::actions::dsl::*;
        let db = self.db_con.lock().unwrap();
        let res = insert_into(actions)
            .values((
                guild_id.eq(attack.get_guild().0 as i64),
                user_id.eq(attack.get_user().0 as i64),
                reactions.eq(attack.get_emotes()),
                expiration.eq(attack.get_expiration()),
            ))
            .execute(&*db);

        if let Err(e) = res {
            println!("Error inserting attack {:?}! {}", attack, e);
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::actions::dsl::*;

        let time = SystemTime::now();

        // Grab the results of the query and minimize the scope of the db lock
        // Looking for either a matching or null author in the proper guild
        let results = {
            let db = self.db_con.lock().unwrap();
            actions
                .filter(guild_id.eq(msg.guild_id.expect("No guild ID for message").0 as i64))
                .filter(user_id.eq(msg.author.id.0 as i64).or(user_id.is_null()))
                .filter(expiration.is_null().or(expiration.gt(time)))
                .load::<Action>(&*db)
                .expect("Query Failed")
        };

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
            } else {
                // No regex, so just add to list if not conflicting
                reaction_set.add_reactions(&action.reactions);
            }
        }

        // Go through all the reactions and react to the message appropriately
        for reaction in reaction_set.get_reaction_str().chars() {
            if let Err(why) = msg.react(ctx.http.clone(), reaction).await {
                println!("Error reacting to message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "attack" => {
                    let mut builder = Attack::builder();
                    builder = builder.set_guild(command.guild_id.unwrap());
                    for entry in &command.data.options {
                        match entry.name.as_ref() {
                            "user" => {
                                if let Some(ApplicationCommandInteractionDataOptionValue::User(
                                    user,
                                    _,
                                )) = &entry.resolved
                                {
                                    builder = builder.set_user(user.id)
                                }
                            }
                            "emotes" => {
                                if let Some(ApplicationCommandInteractionDataOptionValue::String(
                                    s,
                                )) = &entry.resolved
                                {
                                    builder = builder.set_emotes(s)
                                }
                            }
                            "duration" => {
                                if let Some(
                                    ApplicationCommandInteractionDataOptionValue::Integer(int),
                                ) = &entry.resolved
                                {
                                    builder = builder.set_expiration(*int as u64)
                                }
                            }
                            _ => println!("Unexpected entry name: {}", entry.name),
                        }
                    }
                    if let Some(attack) = builder.build() {
                        self.attack(attack);
                        "Attacked".to_string()
                    } else {
                        "Didn't work".to_string()
                    }
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let guild_id = GuildId(908915854484308018);

        // Commands for the bot testing server
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("attack")
                    .description("Set an attack rule")
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user to attack")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("emotes")
                            .description("List of emotes to attack with")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("duration")
                            .description("Length of attack in minutes")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(1)
                            .max_int_value(1440)
                            .required(true)
                    })
            })
        })
        .await;

        println!(
            "Guild {} has the following guild commands: {:#?}",
            guild_id, commands
        );

        // Commands for all servers
        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("attack")
                    .description("Set an attack rule")
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user to attack")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("emotes")
                            .description("List of emotes to attack with")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("duration")
                            .description("Length of attack in minutes")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(1)
                            .max_int_value(1440)
                            .required(true)
                    })
            })
        })
        .await;
        println!(" Global commands: {:#?}", commands);
        println!("{} is connected!", ready.user.name);
    }
}
