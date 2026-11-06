# Tensei Island

A Dynamic Island for Windows 10 and 11 that stays small. It sits at the top of the screen and shows what is
going on: the song that is playing (Spotify, Pear, browsers, any player in the Windows media card), what Claude
Code is doing in your terminals, your Claude plan usage, a focus timer, and quick system readings. Hover to open
it; it closes when you move away.

Built with Tauri and Svelte on the WebView2 runtime that Windows already has: a few MB to install, no bundled
browser, nothing running in the background that does not need to.

![Open island showing the music page](docs/screenshots/09-open-music.jpg)

## What it does

- **Music**: cover, title, synced lyrics, seek, shuffle, repeat, volume. A short peek when the track changes or
  the volume moves. The accent colour follows the album art.
- **Pear Desktop**: likes, dislikes, Pear's own volume and exact progress through Pear's API Server plugin.
- **Claude Code**: live activity per session (reading, editing, running), Allow or Deny permission requests from
  the island, a chime and a summary when it finishes, and your 5-hour and weekly limits as a ring.
- **Shelf**: drop files on the island to keep them handy, clipboard history, a quick note.
- **Focus**: a timer that counts down in the island while you work.
- **Home**: clock, weather, Wi-Fi and Bluetooth toggles, battery.
- **Looks**: Notch, Pill or Line; 15 themes plus your own; 13 bundled typefaces; springy, smooth or minimal motion.

More screens in [docs/screenshots](docs/screenshots) (browser preview with sample data).

## Install

Download the setup .exe (or the portable zip) from the latest GitHub release or the latest Actions run, and run
it. Settings and data live in `%APPDATA%\in.tenseitech.island`.

### Connect Pear Desktop
In Pear: Plugins → API Server → on. In the island: Settings → Music → Connect Pear, then Allow in Pear's dialog.

### Connect Claude Code
Settings → Agents → Connect Claude Code. This adds HTTP hooks to `~/.claude/settings.json` (a backup is kept)
pointing at `127.0.0.1:47821`. Restart running Claude Code sessions.

## Shortcut
Ctrl+Alt+Space opens and closes the island (change it in Settings → General).

## Build

```sh
npm ci
npx tauri dev     # run
npx tauri build   # installer
```

Pushing to GitHub runs `.github/workflows/build.yml` on a Windows runner and uploads the installer and a
portable zip; tagging `v*` publishes a release. See CLAUDE.md for development notes and STATUS.md for what is done.

## Privacy

Media, system readings, clipboard and Claude Code activity stay on your PC. Plan usage asks Anthropic and OpenAI
with the sign-ins already on the PC, only when turned on. Lyrics (LRCLIB) and weather (Open-Meteo) are fetched only
when turned on, sending just the track or place.
