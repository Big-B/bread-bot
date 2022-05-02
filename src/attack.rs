use serenity::model::id::{GuildId, UserId};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Attack {
    guild: GuildId,
    user: UserId,
    emotes: Vec<String>,
    expiration: SystemTime,
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
}

#[derive(Default, Clone)]
pub struct AttackBuilder {
    guild: Option<GuildId>,
    user: Option<UserId>,
    emotes: Option<String>,
    expiration: Option<SystemTime>,
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
        if let Some(expiration) = now.checked_add(Duration::from_secs(expiration)) {
            self.expiration = Some(expiration);
        }
        self
    }

    pub fn build(self) -> Option<Attack> {
        if self.guild.is_none()
            || self.user.is_none()
            || self.emotes.is_none()
            || self.expiration.is_none()
        {
            None
        } else {
            Some(Attack {
                guild: self.guild.unwrap(),
                user: self.user.unwrap(),
                emotes: self.emotes.unwrap().chars().map(|c| c.to_string()).collect(),
                expiration: self.expiration.unwrap(),
            })
        }
    }
}
