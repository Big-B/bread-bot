use anyhow::{anyhow, Context, Result};
use bread_bot::config::Config;
use bread_bot::target::TargetBuilder;
use clap::Parser;
use diesel::insert_into;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serenity::model::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GuildID of the server to target
    #[arg(short, long)]
    guild: u64,

    /// User to target
    #[arg(short, long)]
    user: Option<u64>,

    /// The emotes to apply to messages
    #[arg(short, long)]
    emotes: String,

    /// A time, in minutes, after which the rule will be removed
    #[arg(long)]
    expiration: Option<u64>,

    /// The regex to match against
    #[arg(short, long)]
    regex: Option<String>,
}

fn get_char_map() -> HashMap<char, char> {
    HashMap::from([
        ('a', '🇦'),
        ('b', '🇧'),
        ('c', '🇨'),
        ('d', '🇩'),
        ('e', '🇪'),
        ('f', '🇫'),
        ('g', '🇬'),
        ('h', '🇭'),
        ('i', '🇮'),
        ('j', '🇯'),
        ('k', '🇰'),
        ('l', '🇱'),
        ('m', '🇲'),
        ('n', '🇳'),
        ('o', '🇴'),
        ('p', '🇵'),
        ('q', '🇶'),
        ('r', '🇷'),
        ('s', '🇸'),
        ('t', '🇹'),
        ('u', '🇺'),
        ('v', '🇻'),
        ('w', '🇼'),
        ('x', '🇽'),
        ('y', '🇾'),
        ('z', '🇿'),
    ])
}

fn convert_to_regional_codes(input: &str) -> Result<String, anyhow::Error> {
    let map = get_char_map();
    let mut set = HashSet::new();
    let s: String = input
        .to_ascii_lowercase()
        .chars()
        .map(|x| {
            set.insert(x);
            map[&x]
        })
        .collect();

    // There were duplicates in the set
    if s.len() != set.len() {
        Err(anyhow!("Input ascii had duplicate characters"))
    } else {
        Ok(s)
    }
}

fn main() -> Result<(), anyhow::Error> {
    use bread_bot::schema::actions::dsl::*;
    let args = Args::parse();

    let mut builder = TargetBuilder::default();

    // If the text is all ascii, then it's probably just words intended to be
    // converted to the unicode regional indicator types.
    builder = if args.emotes.is_ascii() {
        builder.set_emotes(&convert_to_regional_codes(&args.emotes)?)
    } else {
        builder.set_emotes(&args.emotes)
    };

    builder = builder.set_guild(GuildId::from(args.guild));

    if let Some(u) = args.user {
        builder = builder.set_user(UserId::from(u));
    }

    if let Some(e) = args.expiration {
        builder = builder.set_expiration(e);
    }

    if let Some(r) = args.regex {
        builder = builder.set_regex(&r);
    }

    let target = builder.build()?;

    // Read in config file
    let mut reader = BufReader::new(
        File::open("/etc/bread-bot.toml")
            .expect("Expected /etc/bread-bot.toml to exist and be readable"),
    );

    // Parse config file
    let mut config_data = String::new();
    reader
        .read_to_string(&mut config_data)
        .expect("Expected valid UTF-8 in config file");

    let config_data: Config = toml::from_str(&config_data).expect("Invalid config file format");
    let mut connection = PgConnection::establish(&config_data.postgres_url)
        .with_context(|| format!("Error connecting to {}", config_data.postgres_url))?;

    insert_into(actions)
        .values((
            guild_id.eq(target.get_guild().0 as i64),
            user_id.eq(target.get_user().map(|x| x as i64)),
            reactions.eq(target.get_emotes()),
            expiration.eq(target.get_expiration()),
            regex.eq(target.get_regex()),
        ))
        .execute(&mut connection)?;

    Ok(())
}
