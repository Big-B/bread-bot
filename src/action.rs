use crate::schema::actions;
use diesel::{Insertable, Queryable};
use std::time::SystemTime;

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = actions)]
pub struct Action {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: Option<i64>,
    pub regex: Option<String>,
    pub reactions: Vec<String>,
    pub expiration: Option<SystemTime>,
}
