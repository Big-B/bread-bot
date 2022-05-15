use regex::Regex;
use serenity::model::id::{GuildId, UserId};
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Target {
    guild: GuildId,
    user: Option<i64>,
    emotes: Vec<String>,
    expiration: SystemTime,
    regex: Option<String>,
}

impl Target {
    pub fn builder() -> TargetBuilder {
        TargetBuilder::default()
    }

    pub fn get_guild(&self) -> GuildId {
        self.guild
    }

    pub fn get_user(&self) -> Option<i64> {
        self.user
    }

    pub fn get_emotes(&self) -> &[String] {
        &self.emotes
    }

    pub fn get_expiration(&self) -> SystemTime {
        self.expiration
    }

    pub fn get_regex(&self) -> Option<&String> {
        self.regex.as_ref()
    }
}

#[derive(Debug)]
pub enum TargetBuilderError {
    MissingUserAndRegex,
    BadRegex(regex::Error),
    EmptyField(String),
}

impl fmt::Display for TargetBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TargetBuilderError::MissingUserAndRegex => write!(f, "Missing argument"),
            TargetBuilderError::BadRegex(_) => write!(f, "Regex was invalid"),
            TargetBuilderError::EmptyField(s) => write!(f, "{}", s),
        }
    }
}

impl Error for TargetBuilderError {}

#[derive(Default, Clone)]
pub struct TargetBuilder {
    guild: Option<GuildId>,
    user: Option<i64>,
    emotes: Option<String>,
    expiration: Option<SystemTime>,
    regex: Option<String>,
}

impl TargetBuilder {
    pub fn set_guild(mut self, gid: GuildId) -> TargetBuilder {
        self.guild = Some(gid);
        self
    }

    pub fn set_user(mut self, uid: UserId) -> TargetBuilder {
        self.user = Some(uid.0 as i64);
        self
    }

    pub fn set_emotes(mut self, emotes: &str) -> TargetBuilder {
        self.emotes = Some(emotes.to_owned());
        self
    }

    pub fn set_expiration(mut self, expiration: u64) -> TargetBuilder {
        let now = SystemTime::now();
        if let Some(expiration) = now.checked_add(Duration::from_secs(expiration * 60)) {
            self.expiration = Some(expiration);
        }
        self
    }

    pub fn set_regex(mut self, regex: &str) -> TargetBuilder {
        self.regex = Some(regex.to_owned());
        self
    }

    pub fn build(self) -> Result<Target, TargetBuilderError> {
        if self.user.is_none() && self.regex.is_none() {
            return Err(TargetBuilderError::MissingUserAndRegex);
        }
        if self.guild.is_none() {
            return Err(TargetBuilderError::EmptyField(
                "No Guild provided".to_string(),
            ));
        }
        if self.emotes.is_none() {
            return Err(TargetBuilderError::EmptyField(
                "No Emotes provided".to_string(),
            ));
        }
        if self.expiration.is_none() {
            return Err(TargetBuilderError::EmptyField(
                "No Expiration provided".to_string(),
            ));
        }
        if self.regex.is_some() {
            if let Err(e) = Regex::new(self.regex.as_ref().unwrap()) {
                return Err(TargetBuilderError::BadRegex(e));
            }
        }

        Ok(Target {
            guild: self.guild.unwrap(),
            user: self.user,
            emotes: self
                .emotes
                .unwrap()
                .chars()
                .map(|c| c.to_string())
                .collect(),
            expiration: self.expiration.unwrap(),
            regex: self.regex,
        })
    }
}
