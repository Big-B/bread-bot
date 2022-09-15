# bread-bot

Silly discord bot that will react to messages as defined in an associated
postgres database.

# Dependencies

* rust
* postgres
* [diesel](https://diesel.rs/)

# Setup

Start up and configure postgres (create a database, users, etc). Ensure
`/etc/bread-bot.toml` is filled out properly. The provided example is for a
database with a user `bread-bot` and a database `bread`, and uses the discord
provided token and application ID provided for your bot in the [developer
portal](https://discord.com/developers/applications). Run `diesel migration
run`. Start `bread-bot`. An example system file is provided to run as a service
with systemd.

# Adding Rules

`bread-bot` utilizes a database containing a single table:
`id | guild_id | user_id | regex | reactions | expiration`

Adding rules is done through SQL commands directly to the postgres database
currently.

A valid rule needs to have a guild ID. Adding a user ID will target only that
user. Adding a [regex](https://docs.rs/regex/latest/regex/index.html#syntax)
will target only messages for which the regex match. If a message passes through
all the filters, the reactions will be applied to the message. However, if
multiple messages apply, any messages containing duplicate reactions will be
dropped (since duplicates won't be displayed again as reactions). Expiration
dates can be added, and any rule that expires will be automatically removed from
the table.
