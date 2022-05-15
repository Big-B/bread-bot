use regex::Regex;
use serenity::model::id::{GuildId, UserId};
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Attack {
    guild: GuildId,
    user: UserId,
    emotes: Vec<String>,
    expiration: SystemTime,
    regex: Option<String>,
}

impl Attack {
    pub fn builder() -> AttackBuilder {
        AttackBuilder::default()
    }

    pub fn get_guild(&self) -> GuildId {
        self.guild
    }

    pub fn get_user(&self) -> UserId {
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
pub enum AttackBuilderError {
    BadRegex(regex::Error),
    EmptyField(String),
}

impl fmt::Display for AttackBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AttackBuilderError::BadRegex(_) => write!(f, "Regex was invalid"),
            AttackBuilderError::EmptyField(s) => write!(f, "{}", s),
        }
    }
}

impl Error for AttackBuilderError {}

#[derive(Default, Clone)]
pub struct AttackBuilder {
    guild: Option<GuildId>,
    user: Option<UserId>,
    emotes: Option<String>,
    expiration: Option<SystemTime>,
    regex: Option<String>,
}

impl AttackBuilder {
    pub fn set_guild(mut self, gid: GuildId) -> AttackBuilder {
        self.guild = Some(gid);
        self
    }

    pub fn set_user(mut self, uid: UserId) -> AttackBuilder {
        self.user = Some(uid);
        self
    }

    pub fn set_emotes(mut self, emotes: &str) -> AttackBuilder {
        self.emotes = Some(emotes.to_owned());
        self
    }

    pub fn set_expiration(mut self, expiration: u64) -> AttackBuilder {
        let now = SystemTime::now();
        if let Some(expiration) = now.checked_add(Duration::from_secs(expiration * 60)) {
            self.expiration = Some(expiration);
        }
        self
    }

    pub fn set_regex(mut self, regex: &str) -> AttackBuilder {
        self.regex = Some(regex.to_owned());
        self
    }

    pub fn build(self) -> Result<Attack, AttackBuilderError> {
        if self.guild.is_none() {
            return Err(AttackBuilderError::EmptyField(
                "No Guild provided".to_string(),
            ));
        }
        if self.user.is_none() {
            return Err(AttackBuilderError::EmptyField(
                "No User provided".to_string(),
            ));
        }
        if self.emotes.is_none() {
            return Err(AttackBuilderError::EmptyField(
                "No Emotes provided".to_string(),
            ));
        }
        if self.expiration.is_none() {
            return Err(AttackBuilderError::EmptyField(
                "No Expiration provided".to_string(),
            ));
        }
        if self.regex.is_some() {
            if let Err(e) = Regex::new(self.regex.as_ref().unwrap()) {
                return Err(AttackBuilderError::BadRegex(e));
            }
        }

        Ok(Attack {
            guild: self.guild.unwrap(),
            user: self.user.unwrap(),
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
