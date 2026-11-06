# Architecture

```
┌──────────────── island window (transparent, 760×560, top centre) ────────────────┐
│  Svelte UI: IslandApp → CompactView | PeekView | ApprovalCard | Expanded(pages)   │
│  reports the island's rectangle ──► island_set_rect                               │
└───────────────────────────────────────────────────────────────────────────────────┘
          ▲ events (app.emit)                         │ commands (invoke)
          │                                           ▼
┌──────────────────────────── Rust (src-tauri/src) ─────────────────────────────────┐
│ island.rs   cursor poll → click-through on/off, hover, full screen, maximized     │
│ games.rs    front window → game intro, smart hide, play time (no extra thread)    │
│ quiet.rs    Windows Do not disturb (registry change notification, no polling)     │
│ media.rs    GSMTC sessions (event driven) → media://state; media_control          │
│ agents.rs   127.0.0.1:47821 HTTP hook server → agents://state|approval|done       │
│ quotas.rs   Claude / Codex usage every N min → quotas://update                    │
│ system.rs   CPU/RAM/net (only while watched) → system://stats; power://changed   │
│ clipboard.rs message-only window listener → clipboard://changed                   │
│ volume.rs   Core Audio callback → volume://changed; volume_set/mute               │
│ radios.rs   Windows.Devices.Radios + WLAN SSID                                     │
│ shell.rs    autostart (HKCU Run), open/reveal paths, open allowed URLs            │
│ config.rs   settings.json + data.json in %APPDATA%\in.tenseitech.island           │
└───────────────────────────────────────────────────────────────────────────────────┘
Pear Desktop and LRCLIB / Open-Meteo are reached from the frontend with fetch/WebSocket.
```

## Why it stays light

- Tauri + WebView2 (already on Windows 10/11): no bundled browser. Release profile: LTO, `opt-level = "s"`, stripped.
- No polling where Windows has events: media sessions, clipboard and volume are callbacks.
- The cursor poll is the only always-on loop: 25 Hz away from the island, 60 Hz near it, and it only calls into
  the window when the inside/outside state changes.
- CPU, memory and network are read only while a page or view shows them (`system_watch` reference count).
- The frontend clock ticks every 15 s at rest and 4 Hz only while something live is on screen.
- No infinite CSS animations in the island. Each frame of one recomposites the whole transparent window
  (measured 11–21% of a core for a spinner and four equalizer bars). Looping decorations step together on
  the shared 8 fps `beat` store (~3%), which stops when none is on screen and while the island is hidden.
  Marquees slide out and back once, then rest.
- Settings → General → Graphics: software compositing (`--disable-gpu`) drops the GPU process, ~120 MB
  private memory instead of ~185 MB, for ~40% more CPU during an open/close morph.
- Fonts: Latin subsets only (~390 KB total), bundled, no network.

## Island states

`compact` (resting) → `peek` (a few seconds of news) → `approval` (stays until answered) → `drop` (dragging files)
→ `expanded` (pages). The shape morphs with a spring written as CSS `linear()` easing; content cross-fades.
A second live activity sits in a round bubble next to the island, like iOS.

Priority for the resting island: music playing > agent working or waiting > focus timer > music paused in the
last 5 minutes > idle (quota ring or battery, and the clock).

## Settings

`src/lib/stores/settings.svelte.ts` owns the schema and defaults. Rust stores the JSON, broadcasts
`settings://changed` to both windows and reads only the keys it needs (`monitor`, `hideInFullscreen`,
`hookPort`, `hookToken`, `approvalTimeout`, `clipboardHistory`, `clipboardMax`, `quotaClaude`, `quotaCodex`,
`quotaMinutes`, `autostart`, `gameIntro`, `smartHide`, `customGames`, and `graphics` before start-up), each with
its own default.

## Daily behaviour

Calm by default, each adjustable in Settings → General → Behaviour:

- Hover opens after 220 ms, and never while a mouse button is held (dragging a tab or window across the top):
  arriving with the button down counts once it is released. Click-through still follows the cursor at once,
  so files can be dropped.
- Tuck when idle: with nothing live for 20 s the island rests as a thin line (`tuckWhenIdle`, `tuckAfter`).
- A maximized window in front turns the resting island into a thin line (`lineWhenMaximized`, now on).
- Windows Do not disturb (`quiet.rs`): song, volume and charging pop-ups are dropped, chimes are silent, and
  agent news is held and shown as one card when it ends (`respectDnd`). Song-change pop-ups are off by default.
- Starting never steals focus: for 5 s after start, focus that lands on the island without a hover or the
  shortcut goes back to the previous window (Tauri's first show and WebView2's load both activate it).
- State that events only report on change (maximized, hidden, shown) is read once at start with
  `island_state`, since the frontend can finish loading after the first events went out.

## Games

`island.rs` checks the foreground window on every poll tick (a single cheap call) and hands it to
`games::Tracker` only when it changes, plus once a second to notice exits. A game is a known exe, an exe from
`customGames`, or a window covering 60% of the monitor whose exe sits in a store library folder (Steam, Epic,
Riot, Rockstar, EA, Ubisoft, Xbox, GOG, Battle.net), named after its folder. Processes are opened with
`PROCESS_QUERY_LIMITED_INFORMATION` only and closed at once; when anti-cheat refuses even that, the window
title is matched against the known names and the window's existence stands in for the process.

On the first front appearance of a game process: `game://launch`, and `intro_until` keeps the island up for
4.6 s over full screen. While a game is in front and `smartHide` is on, the island is hidden. The open shortcut
calls `island_summon(true)`, which shows it over anything until it closes (`island_summon(false)`). When the
process exits: `game://ended` with the play time (the frontend skips sessions under a minute). While a game hides the island, the frontend holds
agent done/attention news (no chime) and shows it as one "While you were playing" card on `game://active` null.

## Commands and events

Commands: see `invoke_handler` in `src-tauri/src/lib.rs`. Events:
`settings://changed`, `island://hover|toggle|fullscreen|maximized|placed|hidden|shown|quiet`,
`game://launch|ended|active`, `media://state`,
`agents://state|approval|done|attention|error`, `quotas://update`, `system://stats`, `power://changed`,
`clipboard://changed`, `volume://changed`. Types are mirrored in `src/lib/types.ts`.
