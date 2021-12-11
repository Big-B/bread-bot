use crate::schema::actions;
use diesel::{Queryable, Insertable};

#[derive(Queryable, Insertable, Debug)]
#[table_name="actions"]
pub struct Action {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: Option<i64>,
    pub regex: Option<String>,
    pub reactions: Vec<String>,
    pub expiration: Option<i64>,
}
