# Research notes

What was learned from the two reference apps and from Pear Desktop, and how it shaped Tensei Island.

## BrimDeck (open source, Apache-2.0)

- Repo: https://github.com/MoFeng2223/BrimDeck, read at commit `5cbf559` (2026-11-02).
- Stack: C# / WPF on .NET 10, about 47k lines across `BrimDeck`, `BrimDeck.Core` and `BrimDeck.Sensors`.
- Features: AI plan quotas for Claude, Codex, Cursor, Antigravity and others (up to 7 columns); music with
  synced lyrics, equalizer and per-player "full control" patches for NetEase, KuGou, QQ Music and Apple Music;
  system monitor (CPU, GPU, memory, network, temperatures, power); collapsed styles Notch, Capsule and Indicator;
  hides in full screen, turns into the Indicator when a window is maximized; panel themes (Dark, Light,
  One Dark Pro, GitHub, Dracula, Night Owl, Cyberpunk) plus a theme editor.
- How it reads Claude usage: `~/.claude/.credentials.json` → `claudeAiOauth.accessToken`, then
  `GET https://api.anthropic.com/api/oauth/usage` with header `anthropic-beta: oauth-2025-04-20`. Response has
  `five_hour` / `seven_day` (`utilization`, `resets_at`), newer `limits[]` (`kind`, `percent`, `resets_at`,
  `scope.model.display_name`), and `seven_day_opus` / `seven_day_sonnet`.
- How it reads Codex usage: `~/.codex/auth.json` → `tokens.access_token`, `tokens.account_id`, then
  `GET https://chatgpt.com/backend-api/wham/usage` with `ChatGPT-Account-Id`. Response `rate_limit.primary_window`
  / `secondary_window` (`used_percent`, `limit_window_seconds`, `reset_at` or `reset_after_seconds`).
- Lyrics: LRCLIB (`/api/get`, `/api/search`).
- Taken into Tensei Island: the quota endpoints and parsing (rewritten in Rust, credited in NOTICE), the
  Notch / Pill / Line styles, hide-in-full-screen and line-when-maximized ideas, LRCLIB. Not taken: the
  Chinese-player patches (they modify other apps' `libcef.dll`), the WPF code itself.

## TwoCode (paid, closed source)

Studied clean-room: only the public website, the installer's metadata, the file names inside the package and
the user-facing docs it ships (README, changelog, in-app help JSON). No program code was read, decompiled or
reused, and the licence and payment flow were not touched.

- Installer: `TwoCode-Setup.exe` 1.2.0-beta.78, a 159 MB .NET (Mono/.NET PE32) wrapper around `TwoCode.zip`.
- App: Electron 44 (`TwoCode.exe` is 240 MB; 387 MB unpacked with Chromium, ffmpeg, SwiftShader, D3D compilers),
  app code in plain `resources/app/*.cjs|js`, PowerShell helpers for media session, volume and window lookups,
  a C# helper for Windows AI access, an Inter variable font, a bundled VS Code extension.
- Price: €9.99 once for 3 PCs, Stripe checkout, accounts on twocode.nl; beta users before 2026-10-30 are "Founders".
- Features (from its README and help): Claude Code and Codex live activity (files read and edited, commands run),
  Allow / Deny of permission requests from the island, completion chime + colour + Windows notification and history,
  5-hour Claude limit ring, a mascot ("Robo") that can be dragged onto a window to ask the connected AI about a
  screenshot, "fix it for me" edits, Spotify controls and device switching, synced lyrics, file tray with
  phone transfer over QR, clipboard history, a quick note, focus timer and focus guard for distracting sites,
  Wi-Fi and Bluetooth toggles, battery / CPU / RAM, a game mode, Ctrl+Alt+T to open and Ctrl+Alt+1–6 for pages,
  self-update with signed app-only patches.
- Taken into Tensei Island (re-implemented from the feature list, original design and code): agent live
  activity and approvals (through Claude Code's HTTP hooks instead of a bundled helper), completion chime and
  peek, quota ring, file shelf, clipboard history, note, focus timer, Wi-Fi / Bluetooth toggles, global shortcut.
  Not taken: the mascot and its branding, phone transfer, focus guard, Spotify device switching (see STATUS).
- The size difference is the main lesson: Electron ships its own Chromium. Tensei Island uses Tauri, which uses
  the WebView2 runtime that Windows 10/11 already has, so the installer is a few MB.

## Pear Desktop (YouTube Music desktop app, open source)

- Repo: https://github.com/pear-devs/pear-desktop (formerly th-ch/youtube-music), read at commit `3337446`.
- Plugin "API Server" (`src/plugins/api-server`): Hono server, default `0.0.0.0:26538`, CORS enabled,
  auth strategy `AUTH_AT_FIRST` by default.
- Auth: `POST /auth/{clientId}` shows an Allow / Deny dialog in Pear the first time; returns `{ accessToken }`
  (HS256 JWT). Send `Authorization: Bearer <token>` on `/api/*`.
- REST (`/api/v1/…`): `previous`, `next`, `play`, `pause`, `toggle-play`, `like-state` (GET, `{state: LIKE|DISLIKE|INDIFFERENT|null}`),
  `like`, `dislike`, `seek-to` `{seconds}`, `go-back`/`go-forward` `{seconds}`, `shuffle` (GET/POST),
  `repeat-mode` (GET), `switch-repeat` `{iteration}`, `volume` (GET `{state,isMuted}`, POST `{volume}`), `toggle-mute`,
  `fullscreen`, `song` / `song-info`, `queue`, `queue/next`, `search`.
- WebSocket: `ws://host:port/api/v1/ws?token=…`, messages `PLAYER_INFO` (on connect), `VIDEO_CHANGED`,
  `PLAYER_STATE_CHANGED`, `POSITION_CHANGED`, `VOLUME_CHANGED`, `REPEAT_CHANGED`, `SHUFFLE_CHANGED`.
  Song fields: `title`, `artist`, `album`, `imageSrc`, `songDuration`, `elapsedSeconds`, `isPaused`, `videoId`, `url`.
- Pear also appears in Windows media controls, so the island de-duplicates its session when the API is connected.

## Claude Code hooks (https://code.claude.com/docs/en/hooks)

- Handler types include `"http"` with `url`, `timeout`, `headers`; the event JSON is POSTed. A 2xx with an empty
  body means "no decision"; a 2xx JSON body uses the same schema as command hooks. Connection failures are
  non-blocking, so hooks are harmless when the island is closed.
- `PermissionRequest` answer: `{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}`
  or `"behavior":"deny"` with a `message`.
- Default timeout 600 s for http hooks. Common input fields: `session_id`, `cwd`, `hook_event_name`,
  `tool_name`, `tool_input`; `Stop` has `last_assistant_message`.
