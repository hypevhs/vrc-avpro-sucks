# vrc-avpro-sucks

This is a program that runs in the background while you play VRChat on Linux. If you are in a world with a recognized
video player, this program will open `mpv` and, based on the information coming from VRChat's log file, forward commands
to `mpv` in order to keep the video synced with what you'd expect to see ingame.

### Roadmap

#### Done, needs testing

#### To Do

- Prevent seeking (i.e. via a seek line) before the video is fully loaded
- Joining a world where the ingame player has already ended shouldn't replay the video from the beginning
- Add controls (terminal and/or mpv keybinds) for e.g. switching ingame player, resyncing, detach state from VRC player
- Listen to pause and resume events from log
- Prevent overzealous seeks
- Try out in VR mode with wlx-overlay-s's mirror
- Reset state when switching worlds
- If multiple players in the world, pick only one
- When the ingame player is stopped, clear out MPV too