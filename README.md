# DiscordBot


Goals:

- [x] basic story features
- [x] show options and associated text
- [x] images and other attachments
- [ ] emoji validation
- [ ] restart from the middle
- [ ] variables
- [ ] generate text from data, for example a health bar made of heart emojis. Maybe ink can do this without special code though, so we'll see.
- [ ] a way to show which options you would have had, but are unavailable, so people want to play again.

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
ExecStart=/home/pi/discord_story_bot client_ids/client_id.txt stories/story1.ink
Restart=always
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

Then run `./deploy_to_raspberry_pi` (but modify it first so that it matches your raspberry pi's ip address, etc. Also, note that you may want to set up your router so your raspberry pi always has the same ip address.)