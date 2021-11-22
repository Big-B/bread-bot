use regex::Regex;
use serenity::model::prelude::*;
use std::sync::Arc;
pub struct Action {
    regex: Option<Arc<Regex>>,
    reaction: ReactionType,
}

impl Action {
    pub fn new(regex: Option<Arc<Regex>>, reaction: ReactionType) -> Self {
        Self { regex, reaction }
    }

    pub fn get_regex(&self) -> &Option<Arc<Regex>> {
        &self.regex
    }

    pub fn get_reaction(&self) -> &ReactionType {
        &self.reaction
    }
}
