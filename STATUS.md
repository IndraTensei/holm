# Status and handoff

Last updated: 2026-11-06 (second session: first run on Windows 11, from the CI build).
Read this first, then CLAUDE.md, then docs/ARCHITECTURE.md.

## Where things stand

Version 0.1.0. All planned v0.1 features are written. GitHub Actions builds it on Windows (clippy, tests,
NSIS installer 1.9 MB, portable zip 2.3 MB). The CI build has run on a real Windows 11 desktop (1920×1080,
100% scale) and the core paths work: placement, click-through, hover, shortcut, full-screen hide, Pear,
Claude Code hooks, volume peek. 🧪 below means not yet seen on real Windows.

Local dev on this PC uses Rust's GNU toolchain (rustup, `x86_64-pc-windows-gnu`, no admin) plus MinGW from
winget (`BrechtSanders.WinLibs.POSIX.UCRT`, for `dlltool`): put `%USERPROFILE%\.cargo\bin` and WinLibs'
`mingw64\bin` on PATH, then `npx tauri dev` and `cargo clippy` work. Two gaps: `cargo test` builds but the test
binary will not start (`STATUS_ENTRYPOINT_NOT_FOUND`, a manifest problem with GNU), and the linker warns about
merging manifests. CI's MSVC build is the one that ships and the one that runs tests. For a release-like build:
push a branch, `gh workflow run build.yml --ref <branch>`, `gh run download <id>`.

**Open decision for the owner:** the default for Settings → General → Graphics (see Known risks).

## Feature checklist

Legend: ✅ written and checked as far as possible here · 🧪 needs a first run on Windows · ⬜ not started

| Area | Feature | State |
| --- | --- | --- |
| Window | Transparent, top-centre, DPI-aware placement, multi-monitor choice | ✅ one monitor at 100%; multi-monitor and 125–150% 🧪 |
| Window | Click-through outside the island (cursor poll + `set_ignore_cursor_events`) | ✅ flips in ~15 ms; clicks outside reach the app below |
| Window | Hide during full screen, line style when maximized, keep on top | ✅ full-screen hide and return ~0.4 s, no focus stolen; line style 🧪 |
| Window | Tray menu, single instance, settings window, global shortcut (Ctrl+Alt+Space) | ✅ shortcut opens with focus, closes and returns focus; tray 🧪 |
| Window | Start with Windows (HKCU Run) | ✅ 🧪 |
| Island | Notch / Pill / Line, spring morph, compact → peek → approval → drop → expanded | ✅ (browser) |
| Island | Second live activity bubble | ✅ (browser) |
| Music | Windows media sessions (Spotify, browsers, any SMTC app), art, timeline, controls | ✅ seen with Pear; Spotify and browsers 🧪 |
| Music | Pear Desktop API: auth, WebSocket, like/dislike, volume, shuffle, repeat, seek | ✅ connected, live track shown; each control 🧪 |
| Music | Synced lyrics (LRCLIB) with tap-to-seek, compact lyric line | ✅ (browser with sample lyrics) |
| Music | Accent colour from album art | ✅ (browser) |
| Music | Volume peek from Core Audio callback, system volume slider | ✅ peek verified; slider 🧪 |
| Agents | Claude Code HTTP hook server, sessions, activity log, completion peek + chime | ✅ HTTP hooks accepted; session, live tool and log right; done peek 🧪 |
| Agents | Allow / Deny permission requests from the island | ✅ 🧪 |
| Agents | One-click hook install/uninstall in `~/.claude/settings.json` (backup kept) | ✅ install worked (12 events); uninstall 🧪 |
| Agents | Codex live activity | ⬜ (quota only; see next steps) |
| Usage | Claude 5-hour / weekly ring, Codex windows | ✅ Claude ring shown; last good numbers kept on 429 and cached across restarts; Codex 🧪 |
| System | CPU, memory, network, battery with sparklines; charging peek | ✅ 🧪 |
| System | GPU, temperatures, per-core | ⬜ |
| Shelf | File drop onto the island, open, show in folder | ✅ 🧪 |
| Shelf | Drag files back out of the shelf | ⬜ |
| Shelf | Clipboard history (text, private copies skipped), quick note | ✅ 🧪 |
| Widgets | Focus timer, weather (Open-Meteo + city search), Wi-Fi / Bluetooth toggles + SSID | ✅ 🧪 |
| Look | 15 themes + custom theme editor, 13 bundled fonts, motion presets, size | ✅ (browser) |
| Build | GitHub Actions: check, clippy, test, NSIS installer + portable zip, release on `v*` tag | ✅ green on main and branches (~20 min); `v*` release not run |
| Perf | Looping decorations on a shared 8 fps beat; Graphics setting (hardware or software) | ✅ measured, see log |
| Games | Intro ("Opening Grand Theft Auto V") with the open/hide keys, then smart hide while in front | ✅ verified with stand-in games; real games 🧪 |
| Games | Recognition: known exes, store library folders, user list (Settings → Games); title fallback | ✅ known and custom verified; store folders and anti-cheat fallback 🧪 |
| Games | Open shortcut summons the island over a game; hide shortcut (Ctrl+Alt+H); play time on quit | ✅ verified |

## Next steps, in order

1. Merge PR #1 (`perf/stepped-loops`), then PR #2 (`feat/game-mode`, stacked on it); choose the Graphics
   default (hardware for now).
2. Real games: launch GTA V, Valorant (Vanguard: checks the title fallback), a Steam game (library folder
   detection), something in exclusive full screen (the intro may not draw over it). Tune the exe list.
3. Finish the 🧪 items by hand: tray menu, settings window, an Allow/Deny prompt (ask `claude` to run
   `npm install` without bypass mode), the done peek and chime, Spotify and a YouTube tab, each Pear control,
   line style with a maximized window, a second monitor, 125–150% scaling.
4. Install the setup .exe for a clean user; check Start with Windows.
5. Remaining always-on cost: ~3% of a core while an agent works (8 fps beat plus the 4 Hz clock). Next: tick
   the agent's elapsed time at 1 Hz (it only shows seconds) and try 6 fps.
6. Memory: ~185 MB (hardware) or ~120 MB (software) private across 7 WebView2 processes. Next ideas: shrink
   the window to the island while compact (less to composite; risk of flicker on resize), and find out what
   the second renderer is for.
7. Then the ⬜ items: Codex live activity (tail `~/.codex/sessions/**/rollout-*.jsonl`, or Codex `notify`),
   drag-out from the shelf (`tauri-plugin-drag`), GPU/temperature readings (PDH counters), Windows toast for
   agent completion when the island is hidden, Bluetooth device name, calendar (ICS) on Home, self-update
   (`tauri-plugin-updater` + signed releases), code signing.

## Known risks and things to watch

- **Click-through** works as built. While the cursor is on the island the whole 760×560 window takes clicks;
  it gives them back within ~15 ms of leaving the island's rectangle.
- **Animations are expensive here.** Every animation frame makes WebView2 recomposite the whole transparent
  window. No `infinite` CSS animations or long transitions on the resting island: use the `beat` store.
- **Graphics default (owner's call).** Software compositing saves ~65 MB and was no worse at rest, but costs
  ~40% more CPU per open/close and may look less smooth. Hardware stays the default until someone decides.
- **Show without focus.** Full-screen hide uses `ShowWindow(SW_HIDE / SW_SHOWNOACTIVATE)` directly on the HWND,
  outside tao's state tracking. If tao re-shows or re-hides the window unexpectedly, route through
  `window.hide()/show()` and re-check focus stealing.
- **Hook field names.** `UserPromptSubmit` and `PreToolUse` parse correctly from live Claude Code. Not seen live
  yet: `notification_type`, `last_assistant_message` and the `PermissionRequest` input. Check one real payload (`island.log`
  sits next to settings.json in `%APPDATA%\in.tenseitech.island`).
- **Windows 11 24H2 Wi-Fi name** needs location access for desktop apps; without it the chip says "Wi-Fi".
- **Claude usage endpoint** is not a public API and rate-limits hard (five restarts in a few minutes got a 429).
  Readings are now cached in data.json and kept through failures; repeated 429s back off up to an hour.
- **Pear on another PC.** The CSP allows `127.0.0.1` and `localhost` only; a LAN host needs a CSP entry.
- Hook token and Pear token live in `settings.json` in the user's AppData (plain text, local only).
- No licence chosen yet (owner's decision). Third-party notices are in NOTICE.

## Session log

- 2026-11-06: Researched BrimDeck (source), TwoCode (clean-room: site, installer metadata, file list, shipped
  docs), Pear API Server (source) and Claude Code hooks (docs). Built backend modules, Svelte UI, settings
  window, themes, CI. Verified: `cargo clippy` for windows-msvc clean; `svelte-check` 0 errors; `vite build`;
  20 UI states + 4 settings pages screenshotted and reviewed (fixed lyric scroll, volume overflow, idle ring
  reading as a spinner, log/prompt styling clash). Lyrics and weather go through a Rust `fetch_json` command
  with a host allowlist so CORS cannot break them; first window reveal goes through Tauri's `show()`.
- 2026-11-06 (session 2, Windows 11 desktop): the CI run on main was already green, so the portable build was
  run directly. Checked with Win32 probes (PowerShell with user32 through Add-Type, screenshots cropped to the
  island): placement, click-through timing, hover open, Ctrl+Alt+Space with focus hand-back, full-screen hide
  and return, volume peek, Pear live track, Claude Code HTTP hooks (installed earlier by the owner) feeding the
  Agents page. Found 13–29% of a core at rest with music playing and an agent working: WebView2's GPU process
  recompositing the transparent window for infinite CSS animations. Added the shared 8 fps `beat` store
  (equalizer, agent spinner and pulse), a one-shot marquee, and a paused beat while hidden. Measured on the CI
  build with a synthetic `UserPromptSubmit` hook (session `perf-probe`) keeping the spinner on: 11–21% down to
  ~3%. Tried WebView2 flags through `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`: `--disable-gpu` gave 120 MB vs
  185 MB, so added Settings → General → Graphics (Rust applies it before start-up). Quotas: last good numbers
  kept on failure, cached across restarts, 429 back-off. Branch `perf/stepped-loops`.
- 2026-11-06 (session 3): game mode. New `games.rs`: front-window based detection (known exes, store library
  folders, user list, title fallback for anti-cheat), intro peek `GameIntro.svelte`, smart hide, `island_summon`
  for the open shortcut over games and full screen, hide shortcut, play time when a game closes, Settings →
  Games. Window visibility is now applied only on change (`refresh`, `island://shown`). Set up local Rust (GNU)
  and verified end to end in `npx tauri dev` with stand-in games: copies of powershell.exe named `GTA5.exe` (full
  screen, 70 s) and `questy.exe` (custom list, windowed, 16 s) running a WinForms window. Intro within ~1.7 s of
  launch, hidden after it, summon/close hands focus back to the game, back on quit, play-time card only for the
  70 s run, Ctrl+Alt+H hides and shows.
