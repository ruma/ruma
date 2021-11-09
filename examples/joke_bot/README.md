A simple bot to demonstrate `ruma-client` functionality. Tells jokes when you ask for them.

# Note on dependency versions

This example was written against pre-release versions of `ruma` and
`ruma-client-api`. Check the comments in the `[dependencies]` section of
[`Cargo.toml`](Cargo.toml) for more information.

# Usage

Create a file called `config` and populate it with the following values in `key=value` format:

- `homeserver`: Your homeserver URL.
- `username`: The Matrix ID for the bot.
- `password`: The password for the bot.

For example:

```ini
homeserver=https://example.com:8448/
username=@user:example.com
password=yourpassword
```

You will need to pre-register the bot account; it doesn't do registration
automatically. The bot will automatically join rooms it is invited to though.

Finally, run the bot (e.g. using `cargo run`) from the same directory as your
`config` file. The bot should respond to the request "Tell me a joke" in any
channel that it is invited to.
