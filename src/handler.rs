use crate::action::Action;
use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::UserId, prelude::*},
    prelude::*,
};
use std::collections::HashMap;

pub struct Handler {
    map: HashMap<UserId, Vec<Action>>,
    list: Vec<(Regex, ReactionType)>,
}

impl Handler {
    pub fn new(map: HashMap<UserId, Vec<Action>>, list: Vec<(Regex, ReactionType)>) -> Self {
        Self { map, list }
    }

    fn get_reaction_for_user(&self, id: UserId, msg: &Message) -> Vec<ReactionType> {
        let mut vec = Vec::new();
        if let Some(actions) = self.map.get(&id) {
            // Split into regex and non-regex reactions
            let (r, mut nr): (Vec<&Action>, Vec<&Action>) =
                actions.iter().partition(|&a| a.get_regex().is_some());

            // Collect all the reactions that have matching regexes
            vec = r
                .iter()
                .filter(|a| a.get_regex().as_ref().unwrap().is_match(&msg.content))
                .map(|a| a.get_reaction().clone())
                .collect();

            // Append all the reactions with no associated regex
            vec.extend(nr.iter_mut().map(|a| a.get_reaction().clone()));
        }
        vec
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Set all reactions associated with this user
        for r in self.get_reaction_for_user(msg.author.id, &msg) {
            if let Err(why) = msg.react(ctx.http.clone(), r.clone()).await {
                println!("Error reacting to message: {:?}", why);
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
