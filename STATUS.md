# Status and handoff

Last updated: 2026-11-06 (first build session, Claude in a Linux sandbox).
Read this first, then CLAUDE.md, then docs/ARCHITECTURE.md.

## Where things stand

Version 0.1.0. All planned v0.1 features are written. Nothing has run on Windows yet: the sandbox had no
Windows machine and could not download the .NET SDK or Rust's Windows std. Rust was type-checked and
clippy-clean against `x86_64-pc-windows-msvc` (std built from source, see CLAUDE.md); the frontend
type-checks, builds, and was reviewed in headless Chrome with mock data (`scripts/shots.py`).

**First job for whoever picks this up on Windows: `npm ci && npx tauri dev`, then walk the checklist below.**

## Feature checklist

Legend: ✅ written and checked as far as possible here · 🧪 needs a first run on Windows · ⬜ not started

| Area | Feature | State |
| --- | --- | --- |
| Window | Transparent, top-centre, DPI-aware placement, multi-monitor choice | ✅ 🧪 |
| Window | Click-through outside the island (cursor poll + `set_ignore_cursor_events`) | ✅ 🧪 highest risk |
| Window | Hide during full screen, line style when maximized, keep on top | ✅ 🧪 |
| Window | Tray menu, single instance, settings window, global shortcut (Ctrl+Alt+Space) | ✅ 🧪 |
| Window | Start with Windows (HKCU Run) | ✅ 🧪 |
| Island | Notch / Pill / Line, spring morph, compact → peek → approval → drop → expanded | ✅ (browser) |
| Island | Second live activity bubble | ✅ (browser) |
| Music | Windows media sessions (Spotify, browsers, any SMTC app), art, timeline, controls | ✅ 🧪 |
| Music | Pear Desktop API: auth, WebSocket, like/dislike, volume, shuffle, repeat, seek | ✅ 🧪 (needs Pear with API Server on) |
| Music | Synced lyrics (LRCLIB) with tap-to-seek, compact lyric line | ✅ (browser with sample lyrics) |
| Music | Accent colour from album art | ✅ (browser) |
| Music | Volume peek from Core Audio callback, system volume slider | ✅ 🧪 |
| Agents | Claude Code HTTP hook server, sessions, activity log, completion peek + chime | ✅ 🧪 |
| Agents | Allow / Deny permission requests from the island | ✅ 🧪 |
| Agents | One-click hook install/uninstall in `~/.claude/settings.json` (backup kept) | ✅ 🧪 |
| Agents | Codex live activity | ⬜ (quota only; see next steps) |
| Usage | Claude 5-hour / weekly ring, Codex windows | ✅ 🧪 |
| System | CPU, memory, network, battery with sparklines; charging peek | ✅ 🧪 |
| System | GPU, temperatures, per-core | ⬜ |
| Shelf | File drop onto the island, open, show in folder | ✅ 🧪 |
| Shelf | Drag files back out of the shelf | ⬜ |
| Shelf | Clipboard history (text, private copies skipped), quick note | ✅ 🧪 |
| Widgets | Focus timer, weather (Open-Meteo + city search), Wi-Fi / Bluetooth toggles + SSID | ✅ 🧪 |
| Look | 15 themes + custom theme editor, 13 bundled fonts, motion presets, size | ✅ (browser) |
| Build | GitHub Actions: check, clippy, test, NSIS installer + portable zip, release on `v*` tag | ✅ (not run yet) |

## Next steps, in order

1. **First run on Windows** (`npx tauri dev`). Check: island visible at top centre; hovering opens it; clicks
   outside the island reach the apps below; it hides in a full-screen video; tray and Ctrl+Alt+Space work.
2. Play Spotify or a YouTube tab: art, title, progress, controls, volume peek.
3. Open Pear Desktop, turn on Plugins → API Server, click Connect Pear (Music page or Settings → Music).
4. Settings → Agents → Connect Claude Code; start `claude` in a terminal; check activity, an Allow/Deny prompt
   (e.g. ask it to run `npm install`), and the done peek. If HTTP hooks are not accepted by the installed
   Claude Code, switch Hook type to curl and connect again.
5. Push to GitHub (`IndraTensei/tensei-island`), let the workflow build, install the setup .exe on a clean user.
6. Measure: idle CPU (target ≈0%), memory of the WebView2 processes (target under 120 MB total), installer size.
7. Then the ⬜ items: Codex live activity (tail `~/.codex/sessions/**/rollout-*.jsonl`, or Codex `notify`),
   drag-out from the shelf (`tauri-plugin-drag`), GPU/temperature readings (PDH counters), Windows toast for
   agent completion when the island is hidden, Bluetooth device name, calendar (ICS) on Home, self-update
   (`tauri-plugin-updater` + signed releases), code signing.

## Known risks and things to watch

- **Click-through.** `set_ignore_cursor_events` toggles `WS_EX_TRANSPARENT`; if WebView2 still eats clicks in
  transparent areas, fall back to resizing the window to the island's rectangle (shape anchored top centre).
- **Show without focus.** Full-screen hide uses `ShowWindow(SW_HIDE / SW_SHOWNOACTIVATE)` directly on the HWND,
  outside tao's state tracking. If tao re-shows or re-hides the window unexpectedly, route through
  `window.hide()/show()` and re-check focus stealing.
- **Hook field names.** Parsing is tolerant, but `notification_type`, `last_assistant_message` and the
  `PermissionRequest` input were taken from the docs and not seen live. Check one real payload (`island.log`
  sits next to settings.json in `%APPDATA%\in.tenseitech.island`).
- **Windows 11 24H2 Wi-Fi name** needs location access for desktop apps; without it the chip says "Wi-Fi".
- **Claude usage endpoint** is not a public API; it can change. Errors show in the Agents page instead of numbers.
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
