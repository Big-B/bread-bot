use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::ReactionType},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id == 239194802002853898 {
            if let Err(why) = msg
                .react(ctx.http, ReactionType::Unicode("\u{1F35E}".to_string()))
                .await
            {
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
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
