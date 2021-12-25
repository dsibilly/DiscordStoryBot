# DiscordBot


Goals:

v0.2

- [x] basic story features
- [x] show options and associated text
- [x] images and other attachments
- [x] play only one game at a time, and have a stop command
- [x] timer tools
  - [x] support time scales that are larger than one minute (time formatting)
  - [x] and tick the timer appropriately depending on time scale.
  - [x] and have a way to set these when playing a story
- [x] stories as their own directories, automatically imported
- [x] hide test stories, or make them their own category

v1.0

- [ ] grab the story title from the tag (if there is one)
  - display this in the `!play` list, but still use the filename as they key
- [ ] emoji validation
- [ ] verify that filenames don't have spaces in them, or other bad special characters
- [ ] error handling with messages (when applicable) to the user, otherwise nice logs
- [ ] restart from the middle
- [ ] pause/resume commands
- [ ] variables, logic, and conditionals
- [ ] generate text from data, for example a health bar made of heart emojis. Maybe ink can do this without special code though, so we'll see.

v2.0

- [ ] a way to show which options you would have had, but are unavailable, so people want to play again.
  -  maybe this is a summary of the game, or saying "X branches were not taken", or "found 3/5 endings".

See [the runner readme](runner/README.md) to see which parts of the .ink format are supported.


## Deployment

### To run locally:

Run `cargo run -- -help`, which will tell you do to something like `cargo run -- client_ids/client_id.txt stories/story1.ink`.

### For cross-compiling from WSL/ubuntu to raspberry pi:

Run these:

`rustup target add arm-unknown-linux-gnueabi`

`sudo apt install build-essential`

`sudo apt install gcc-arm-linux-gnueabi`

Set up ssh keys with your raspberry pi, so you don't have to type your password many times.

Make this file (with sudo) `/lib/systemd/system/discord-story-bot.service`, with these contents:

```
[Unit]
Description=Discord Story Bot
After=multi-user.target

[Service]
WorkingDirectory=/home/pi
ExecStart=/home/pi/discord_story_bot client_ids/client_id.txt
Restart=always
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

Then run `./deploy_to_raspberry_pi` (but modify it first so that it matches your raspberry pi's ip address, etc. Also, note that you may want to set up your router so your raspberry pi always has the same ip address.)