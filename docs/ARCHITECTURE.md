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
`quotaMinutes`, `autostart`), each with its own default.

## Commands and events

Commands: see `invoke_handler` in `src-tauri/src/lib.rs`. Events:
`settings://changed`, `island://hover|toggle|fullscreen|maximized|placed|hidden`, `media://state`,
`agents://state|approval|done|attention|error`, `quotas://update`, `system://stats`, `power://changed`,
`clipboard://changed`, `volume://changed`. Types are mirrored in `src/lib/types.ts`.
