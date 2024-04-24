to use, you have to open osu!, and then open rosu-memory, then run this application.

you also have to create the `bot_config.toml` configuration file, with this structure

```toml
[twitch]
name = "<put bot name here>"
token = "<put twitch token for bot here>"
prefix = "!"
channel = "<twitch channel to run bot in>"

[osu]
beatmap_requests = false
server = "irc.ppy.sh"
name = "<osu player name>"
player = "<osu player name>"
password = "<osu irc password>"
api_key = "<osu api v1 key>"
```

some functionality isn't working yet, such as irc beatmap requests, so i disabled it
