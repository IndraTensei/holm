# Tensei Island

A Dynamic Island for Windows 10 and 11 that stays small. It sits at the top of the screen and shows what is
going on: the song that is playing (Spotify, Pear, browsers, any player in the Windows media card), what Claude
Code is doing in your terminals, your Claude plan usage, a focus timer, and quick system readings. Hover to open
it; it closes when you move away.

Built with Tauri and Svelte on the WebView2 runtime that Windows already has: a few MB to install, no bundled
browser, nothing running in the background that does not need to.

![Open island showing the music page](docs/screenshots/09-open-music.jpg)

## What it does

- **Music, no setup**: Spotify, Apple Music, TIDAL, Deezer, Amazon Music, VLC, browsers (YouTube, SoundCloud,
  Spotify web) and anything else in the Windows media card. Cover, title, synced lyrics, seek, shuffle, repeat,
  and the playing app's own volume (not the whole PC's). The accent colour follows the album art.
- **Pear Desktop**: also likes, dislikes and exact progress through Pear's API Server plugin.
- **Claude Code**: live activity per session (reading, editing, running), Allow or Deny permission requests from
  the island, a chime and a summary when it finishes, and your 5-hour and weekly limits as a ring.
- **AI tools**: finds the coding agents on your PC (Claude Code, Codex, OpenCode, Antigravity, Gemini CLI,
  Copilot CLI, Cline, Cursor Agent, Amp and more), shows which are running and what each used today, read
  from the files each tool already keeps.
- **Games**: an intro when a game starts, out of the way while you play, a hotkey to bring it back, and the
  agent news you missed when you are done.
- **Calm by default**: rests as a thin line when idle and over browser tabs, never opens mid-drag, respects
  Windows Do not disturb, never takes focus.
- **Shelf**: drop files on the island to keep them handy, clipboard history, a quick note.
- **Focus**: a timer that counts down in the island while you work.
- **Home**: clock, weather, Wi-Fi and Bluetooth toggles, battery.
- **Looks**: Notch, Pill or Line; 15 themes plus your own; 13 bundled typefaces; springy, smooth or minimal motion.

More screens in [docs/screenshots](docs/screenshots) (browser preview with sample data).

## Performance

Measured on the optimized build: Windows 11 Pro (build 26200), Intel Core i7-11700 (8 cores, 16 threads),
GeForce GTX 1660 Super, 16 GB, 1920×1080 at 100%, WebView2 156, Graphics set to Hardware (the default).
CPU is the island plus its WebView2 processes, as a share of one core and of the whole processor.

| State | CPU, one core | CPU, whole PC | Memory (private) |
| --- | --- | --- | --- |
| First 10 seconds after start | 7.5% | 0.5% | 210 MB |
| Resting as a line, music playing | 1.9% | 0.1% | 236 MB |
| Resting as the pill, music playing (equalizer moving) | 3.7% | 0.2% | 251 MB |
| Open, with an agent working and music playing (the most it does) | about 28% | 1.7% | 320 MB |

- The open island costs the most because it is a large see-through window redrawn on every update; it only
  runs while your pointer is on it.
- Loops (equalizer, spinner) step together at 8 frames a second instead of 60, and stop when nothing is
  moving, the island is hidden, or a game or full-screen app is in front.
- Memory is mostly WebView2. Task Manager's working set reads higher (about 450 MB) because pages shared
  between those processes are counted once per process. Settings → General → Graphics → Software saves about
  65 MB, at a little more CPU when opening and closing.
- Network: nothing unless turned on. Plan usage is read from the copy Claude Code and Codex keep on the PC,
  and their usage services are asked at most every 5 minutes, only when that copy is old.
- Install size: about 2 MB installer, 5 MB on disk, plus the WebView2 runtime that Windows already has.

## Minimum requirements

- Windows 10 version 1809 or later, or Windows 11, 64-bit (x64).
- Microsoft Edge WebView2 Runtime: built into Windows 11 and present on most Windows 10 PCs; the installer
  fetches it if missing.
- About 250 to 350 MB of free memory for the island (4 GB of RAM works, 8 GB recommended), and any 64-bit
  dual-core processor.
- A DirectX 11 graphics card for Hardware graphics; on older or virtual machines choose Software graphics.
- Internet only for the optional parts: lyrics, weather, plan usage.

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
