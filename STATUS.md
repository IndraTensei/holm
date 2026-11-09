# Status and handoff

The app was called Tensei Island until session 6; it is now **Holm** (repo `IndraTensei/holm`).

Last updated: 2026-11-09 (session 7: releases v0.1.0 to v0.1.2, installer options page, lighter compact lyrics, Make way, public repo).
Read this first, then CLAUDE.md, then docs/ARCHITECTURE.md.

## Where things stand

Version 0.1.4. All planned v0.1 features are written. GitHub Actions builds it on Windows (clippy, tests,
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
| Window | Start with Windows (HKCU Run): Settings → General, the installer's options page, `/AUTOSTART` for silent installs; a moved portable exe re-points it | ✅ 🧪 |
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
| Build | GitHub Actions: check, clippy, test, NSIS installer + portable exe, release on `v*` tag with notes from `docs/releases/<tag>.md` | ✅ green on main and branches (~20 min) |
| Build | Installer: Tauri's NSIS template plus an options page (start with Windows, Start menu, desktop), branded sidebar and header | ✅ compiles; pages 🧪 |
| Perf | Looping decorations on a shared 8 fps beat; Graphics setting (hardware or software) | ✅ measured, see log |
| Games | Intro ("Opening Grand Theft Auto V") with the open/hide keys, then smart hide while in front | ✅ verified with stand-in games; real games 🧪 |
| Games | Recognition: known exes, store library folders, user list (Settings → Games); title fallback | ✅ known and custom verified; store folders and anti-cheat fallback 🧪 |
| Games | Open shortcut summons the island over a game; hide shortcut (Ctrl+Alt+H); play time on quit | ✅ verified |
| Games | Agent news while playing is held (no chime) and shown as "While you were playing" after | ✅ verified |
| Daily | No focus stealing at start; hover never opens mid-drag; tuck to a line when idle or behind a maximized window | ✅ verified (dev build) |
| Daily | Over maximized windows: learns per app (accidental hover opens) whether to rest as a line | ✅ verified (dev build); real-life tuning 🧪 |
| Daily | Respect Windows Do not disturb: chatty pop-ups dropped, chimes off, agent news held | ✅ verified by flipping the registry value; real toggle 🧪 |

## Next steps, in order

1. Merge PR #1 (`perf/stepped-loops`), then PR #2 (`feat/game-mode`, stacked on it); choose the Graphics
   default (hardware for now).
2. Real games: launch GTA V, Valorant (Vanguard: checks the title fallback), a Steam game (library folder
   detection), something in exclusive full screen (the intro may not draw over it). Tune the exe list.
3. Finish the 🧪 items by hand: tray menu, settings window, an Allow/Deny prompt (ask `claude` to run
   `npm install` without bypass mode), the done peek and chime, Spotify and a YouTube tab, each Pear control,
   line style with a maximized window, a second monitor, 125–150% scaling.
4. Run the v0.1.0 setup .exe for a clean user: click through the options page (each box on and off), check
   the Run value, shortcuts and the finish page's run box; uninstall removes the Run value.
5. Remaining always-on cost: ~3% of a core while an agent works (8 fps beat plus the 4 Hz clock). Next: tick
   the agent's elapsed time at 1 Hz (it only shows seconds) and try 6 fps.
6. Memory: ~185 MB (hardware) or ~120 MB (software) private across 7 WebView2 processes. Next ideas: shrink
   the window to the island while compact (less to composite; risk of flicker on resize), and find out what
   the second renderer is for.
7. Then the ⬜ items: Codex live activity (tail `~/.codex/sessions/**/rollout-*.jsonl`, or Codex `notify`),
   drag-out from the shelf (`tauri-plugin-drag`), GPU/temperature readings (PDH counters), Windows toast for
   agent completion when the island is hidden, Bluetooth device name, calendar (ICS) on Home, self-update
   (`tauri-plugin-updater` + signed releases), code signing.
8. macOS and Linux (asked for; README says "planned"). The Svelte UI, settings, lyrics, weather, Claude Code
   hooks (HTTP on 127.0.0.1), plan usage and harness reading are portable already. Per system, re-do the
   Windows-only Rust: the island window (always on top, click-through outside the island, placement under the
   notch on Macs with one; on Linux, Wayland does not let apps place or keep windows on top without
   layer-shell, so X11 and wlroots compositors first), media (macOS has no public now-playing API, and the private MediaRemote one was locked down in
   15.4, so it needs a workaround; Linux has MPRIS over D-Bus, which is easy), per-app volume, Wi-Fi/Bluetooth toggles, full-screen and
   game detection, start at login, and the installer (.dmg signed and notarized; AppImage/.deb). Gate each
   module with `#[cfg(target_os)]` and stub what a system can't do, so features hide rather than break.

## Known risks and things to watch

- **Windows Defender flagged the installed build** (PR #2 CI build, installed with the setup .exe) as
  `Trojan:Win32/Bearfoos.A!ml` on its second launch and quarantined it. `!ml` is a cloud machine-learning verdict,
  not a signature: on-demand scans of the same code (portable exe), PR #1, v0.1.0 and the local debug build all
  come back clean. Likely causes: an unsigned, never-seen-before exe whose behaviour looks spyware-shaped to a
  model (reads the Claude Code OAuth file and calls an API with it, clipboard listener, HKCU Run autostart,
  foreground window/process queries, global hotkeys, edits ~/.claude/settings.json). Release blocker. Fixes,
  best first: sign releases (Azure Trusted Signing or an OV certificate); submit each release to Microsoft as a
  false positive (microsoft.com/wdsi/filesubmission, developer account); consider making the Claude usage ring
  opt-in so a fresh install does not read the credentials file.

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
  sits next to settings.json in `%APPDATA%\in.tenseitech.holm`).
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
  70 s run, Ctrl+Alt+H hides and shows. Agent done/attention news during a game is held without a chime and
  shown as one "While you were playing" card afterwards (verified with a synthetic Stop hook mid-game).
  The PR #2 CI build installed silently (setup.exe /S, per-user, Start menu entry) and passed the same game test,
  then Defender quarantined it on relaunch (see Known risks). Island left stopped pending the owner's decision.
- 2026-11-06 (session 4): daily-use pass. Found and fixed: the island took focus on start (also why it never
  saw a maximized window in front), and the frontend missed state events sent before it loaded (now
  `island_state` at start). Added drag-safe hover, tuck when idle, line behind maximized windows by default,
  Do not disturb (`quiet.rs`), calmer defaults (hover 220 ms, no song pop-ups, softer glow), Settings →
  General → Behaviour. Verified in `npx tauri dev`: focus stays with the user's window through a restart; line
  over a maximized window; plain hover opens from the line; a press held over the island does not open it and
  release does (test pressed only inside its own verified-foreground window: an earlier version of that test
  pressed into the owner's browser because a background script cannot take focus, so never inject clicks
  without checking what is under the cursor); Do not disturb holds a done event and shows it after; tuck after
  20 s in the browser preview. Higgsfield: "Katana" is a claude.ai feature behind the Higgsfield MCP connector
  (https://mcp.higgsfield.ai/mcp), not in the CLI; the video is planned as code-made motion of the real UI.
- 2026-11-06 (session 4, later): the usage ring kept showing "Rate limited" after restarts. Cause: only good
  readings were cached and only their time, so a restart forgot the last numbers and the backoff, and asked
  again at once (many restarts under `tauri dev`). Now the cache keeps the last good reading per provider, the
  next check time and the backoff; Retry-After is honoured; old readings show dimmed with their age. The
  service itself answered 200 when probed, so the 429s are intermittent.
- 2026-11-06 (session 4, later): the thin line over every maximized window was too much (the owner works in
  a maximized T3 Code all day). Replaced with Learn / Thin line / Full (`overMaximized`, default Learn):
  per-app learning from accidental hover opens (see ARCHITECTURE, Daily behaviour). Verified on the real app
  with pointer moves only: full over maximized T3 Code and Vivaldi until learned; three quick hover-and-leaves
  over Vivaldi → "Resting as a line over vivaldi" and a thin line there, saved to settings.json (test entry
  removed afterwards). Found on the way: a stale faded frame of the open island after it shrank (WebView2
  partial redraw); fixed with a full repaint after changes settle, not seen again in the rerun. One hover
  counted twice in the real app (hover flicker while growing); misses within 1.5 s now count once.
- 2026-11-06 (session 4, later): owner feedback round. (1) Vivaldi: the centre tab under the island could not
  be clicked, because the line kept the full pill's hover area. Over a maximized app the line now opens only at
  the top screen edge, the gap above the island counts as inside, and browsers start as a line
  (`browsersAsLine`). Verified with pointer moves: on the tab area the tab gets the hover (Vivaldi's tab
  preview shows), top edge opens, away returns to the line. (2) Spinner looked choppy (an arc stepped 30
  degrees per beat): now eight spokes with a fading trail on the same 8 fps beat. (3) Settings → AI tools:
  detection and today's usage for 15 coding tools (`harness.rs`); real scan here found Claude Code, Codex,
  OpenCode, Antigravity, Copilot CLI and Cline; first scan 3.9 s in a debug build (Claude logs read in full
  once), 51 ms after. (4) No AI attribution in commits or PRs from now on (owner's call); PR descriptions
  cleaned.
- 2026-11-06 (session 4, later): two owner reports. (1) The island opened with the pointer well below it
  (scrolling in T3 Code): closing could leave Rust holding the open island's hover area (measured mid
  animation, transitionend missing). Fixed: measure again once settled, and hover-open needs the pointer
  over the island itself. Verified with pointer moves over a maximized window: no hover at 60 to 250 px.
  (2) Usage was 38 min stale (the usage service answered 429 for a while; it is shared with Claude Code).
  Now Claude Code's own copy (`~/.claude.json` cachedUsageUtilization) and Codex's session logs are read
  every minute (default `quotaMinutes` 1), and a service is asked only when the newest reading is over
  15 min old, at most every 5 min, with per-provider backoff. Verified: Claude reading follows Claude Code's
  copy. Also: vite ignores src-tauri/target (a release build next to tauri dev reloaded every window,
  leaving the settings window blank), Codex window labels ("Monthly", not "43200-minute").
- 2026-11-06 (session 4, later): music for every player without setup. Spotify, browsers, VLC and the rest
  already came through Windows media controls; the gap was volume (the slider moved the whole PC's). Now
  `appvolume.rs` sets the playing app's own mixer volume (GSMTC app id matched to the audio session's
  process); PC volume only when no match. Verified the matching on this PC: Pear ("youtube music") and
  Vivaldi found; Spotify and Chrome not running. "Connect Pear" shows only while Pear plays. Performance for
  the README measured on the optimized build (i7-11700, 16 threads): line 1.9% of a core, pill with music
  3.7%, open with agent and music ~28% (large see-through surface; next: cut the open island's repaints),
  private memory 236 to 320 MB. The first AI tools scan peaked at 560 MB because it read whole logs: Claude
  logs are now streamed line by line with only the needed fields parsed (not re-measured yet).
- 2026-11-06 (session 4, later): open island and memory. Music page ran layout at 60 fps while playing (the
  progress bar transitioned its width every tick): now a stepped transform (41 to 5 ms of work a second).
  Elapsed times tick at 1 Hz (clock.second), spinner and equalizer on their own layers. Open island with an
  agent working: 28% of a core before (hardware, with music), now 5% hardware or 8% software. Memory per
  process: GPU process 112 MB, page 88, browser 42, spare page 19. New default Graphics = Software plus lean
  WebView2 flags (no spare renderer, no background networking or component updates, one renderer for both
  windows): 116 MB private at rest (was 259), 156 MB with settings open; settings verified to render. Also
  Home: long titles ellipsize (grid columns could not shrink), usage rings one line each, "Clear in" spacing.
- 2026-11-06 (session 4, later): the settings window opened blank white from the island's gear (also earlier
  today, wrongly put down to a vite reload): `open_settings` was a synchronous command, and creating a
  WebView2 window from one stalls the new webview's setup. Now async. Also the hover guard from the
  pop-open fix could refuse a flick to the top edge (the page saw no pointer move before the click-through
  flip): the page now asks Rust for the real pointer (`island_cursor`). Verified on the release build: one
  jump to the top edge opens the island, the gear opens a fully drawn settings window.
- 2026-11-06 (session 5): stack merged to main (PR #1, then PR #2 retargeted and merged). Promo video and
  README GIF: `scripts/film.py` drives the browser preview through `?film=1` (mock event hook) and records
  nine scenes with Chromium's screencast. Headless capture tops out near 30 fps, so the page runs at a third
  of real speed (DevTools Animation playback rate plus Playwright's clock) and plays back 3x: ~85 fps of page
  time, encoded at 60. Edited in Higgsfield's sandbox (Katana workflow): a Pillow + ffmpeg compositor with an
  eased top-anchored camera, captions in Bricolage Grotesque and Geist, title and end cards with the app icon,
  cuts on the downbeats of an ElevenLabs Music track (120 BPM, 16.86 credits), CC0 UI sounds, -14 LUFS.
  Outputs: `promo/tensei-island-promo.mp4` (55 s, 1080p60, kept out of git) and `docs/media/demo.gif`
  (17 s loop, 720 px, 2.7 MB) at the top of the README.
- 2026-11-06 (session 6): renamed to **Holm** (a holm is a small island in a river). Product name, window titles,
  exe (`holm.exe`), crate (`holm`, `holm_lib`), identifier `in.tenseitech.holm`, CI artifact and portable zip,
  docs. Upgrades keep working: on first start Holm copies `settings.json` and `data.json` from
  `%APPDATA%\in.tenseitech.island` when it has none of its own, moves the "TenseiIsland" Run value (start with
  Windows) to "Holm" with the new exe path, and still recognises Claude Code hooks marked `src=tensei-island`.
  New logo, "Notch H": a solid H whose counters are the island's notch shape, in Periwinkle `#6F7FF7`, with a
  Bricolage Grotesque wordmark; files and usage notes in `docs/brand/`, app icons regenerated (a pixel-grid cut
  for 32 px and below), mark in the settings sidebar, README header, favicon. Promo pixelation found and fixed:
  headless screencast frames are CSS-sized (960 px) whatever the device scale, so the old clips were upscaled
  twice. `scripts/film.py` now takes 4K screenshots (CDP clip with scale) at 1/8 page speed; the compositor crops
  the 4K frames, so every zoom is sharp. Also fixed "Â·" mojibake in the old end card. Outputs: `promo/holm-promo.mp4`
  (55 s, 1080p60, same soundtrack, kept out of git) and a new `docs/media/demo.gif` (720 px, 3.3 MB).
- 2026-11-09 (session 7): first release, **v0.1.0**. The release ships the setup .exe and the bare portable exe
  (the zip is gone); notes come from `docs/releases/v0.1.0.md`. The installer now uses a copy of Tauri's NSIS
  template (`src-tauri/windows/installer.nsi`, tauri-cli 2.12.4, changes marked `Holm:`; re-apply them when
  upgrading tauri-cli) with an options page after the folder page: start Holm when I sign in (on by default; on
  a reinstall it follows the existing Run value), add to the Start menu, add a desktop shortcut (moved here from
  the finish page). Silent installs skip it; `/AUTOSTART` turns start with Windows on. The uninstaller already
  removed the "Holm" Run value. Sidebar and header bitmaps drawn from the brand SVGs by
  `scripts/installer_art.py`. Kept `installMode: currentUser`: "both" would put a UAC prompt in front of every
  admin user for a per-user tray app. New `shell::repair_autostart`: if the Run value names an exe that no
  longer exists (a moved portable copy), it is pointed at the running exe.
- 2026-11-09 (session 7, later): the resting island's lyric line. Marquee faded its last 14 px even when the
  text fit, so the last word of a line looked cut off; it now fades only while text is hidden, measures
  fractional widths, and has a `once` mode (one quick pass to the end, then stays, with a fade on both edges)
  for lyrics, which change faster than the old slow out-and-back slide could finish. The compact lyric line no
  longer runs the clock at 4 Hz: one timer per line, set for the next line's start, re-armed when the player
  reports a new position. A song with synced lyrics keeps one 240 px width for all its lines, so the island no
  longer animates its width and re-shapes its window on every line. README: resting-island choices, a Using it
  section (tray menu, bubble, settings pages), performance notes.
  WebView2 processes can't be renamed (they are Microsoft's signed msedgewebview2.exe; renaming would mean
  shipping a ~150 MB fixed runtime with no Windows updates). Task Manager's Processes tab already groups them
  under "Holm" (holm.exe's FileDescription); README and Settings → About now say so.
  Licensed MIT (LICENSE, README section, package.json and Cargo.toml) and made the repo public.
  Website holm.indratensei.me: repo homepage, README header and Install, and a Holm website button in Settings → About.
  "Make way" (new default for Over a maximized window): the full island over a maximized app, dropping to a
  line while the pointer is near (Rust's existing near zone, sent as `island://near` on change) and back when it
  leaves; open by pushing to the top edge. Learn, Thin line and Full stay. Preview: `?front=chrome&near=1`.
  README: a Supported systems table (Windows 10 1809+/11 x64; ARM untested; macOS and Linux planned) and a line
  under the header. Port plan is Next step 8.
- 2026-11-09 (session 7, open source): merged branches deleted (and auto-delete on merge turned on),
  Discussions and private vulnerability reporting on. Added CONTRIBUTING, CODE_OF_CONDUCT (Contributor
  Covenant 2.1), SECURITY, CHANGELOG, issue forms (bug, feature) and config, PR template, Dependabot (quarterly, one combined PR; was monthly,
  grouped), CODEOWNERS, .editorconfig; labels for platforms and areas; tracking issues #10 (macOS) and #11
  (Linux). README restructured: badges, links row, Platforms (with the issues), Contributing, Build from
  source. CI caches Tauri's NSIS download and retries bundling once (two 504s from GitHub failed builds today).
- 2026-11-09 (session 7, v0.1.3): Make way flickered with the pointer resting ~125 px below the island: the
  near zone was measured from the current rect, which shrank to the 1 px edge strip when the island made way.
  New `near.rs` (with tests): zone from the last full rect, 48 px slack once near, 250 ms out before leaving.
- 2026-11-09 (session 7, v0.1.4): Make way's near zone shrunk to 40 px either side and 48 px below the full
  island (24 px slack): at 160/120 it stepped aside while the pointer was elsewhere near the top of the screen.
