# bread-bot

Silly discord bot that reacts with bread for every message a particular person
sends.

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
