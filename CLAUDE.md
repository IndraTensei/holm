# Tensei Island — notes for Claude Code

A Dynamic Island for Windows 10/11. Tauri 2 (Rust) + Svelte 5 (TypeScript), WebView2. Windows only.
Start every session by reading STATUS.md, and update its checklist and session log before you finish.

## Commands

```sh
npm ci                     # frontend packages
npm run check              # svelte-check (must be 0 errors, 0 warnings)
npm run dev                # browser preview at :1420 with mock data (no Rust needed)
npx tauri dev              # the real app (Windows)
npx tauri build            # NSIS installer in src-tauri/target/release/bundle/nsis
cargo clippy --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Browser preview scenes (mock data, `src/lib/mock.ts`): `?open=music|agents|home|system|shelf|focus`,
`?peek=done|track|volume|charging`, `?approval=1`, `?drop=1`, `?timer=1`, `?game=<name>`, `?front=<app>` (a maximized app in front), `?ended=1`, `?missed=1`, `?theme=<id>`, `?style=notch|pill|line`,
`?music=off`, `?agents=off`, `?title=lyrics`, `#settings&section=look`, `?section=games#settings`, `?section=tools#settings`. `python3 scripts/shots.py` screenshots them
(needs `npx vite preview --port 4173` and Playwright).

Type-checking the Windows backend from Linux (no Windows std available as a download): put Rust's
`library/` source for your rustc version at `$(rustc --print sysroot)/lib/rustlib/src/rust/library`
(including the `backtrace` submodule), then
`RUSTC_BOOTSTRAP=1 cargo check -Zbuild-std=std,panic_abort --target x86_64-pc-windows-msvc` in `src-tauri`.

## Layout

- `src-tauri/src/*.rs`: one module per integration; each emits events and exposes commands registered in `lib.rs`.
- `src/lib/stores/*.svelte.ts`: one store per integration (Svelte 5 runes classes); `settings.svelte.ts` owns defaults.
- `src/lib/views`: island states. `src/lib/pages`: expanded pages. `src/lib/settings`: settings window sections.
- `src/lib/themes.ts`: themes, fonts and colour math. `src/styles`: base CSS and the bundled font faces.
- `docs/ARCHITECTURE.md`, `docs/RESEARCH.md`: how it fits together and what the reference apps taught.

## Rules of the house

- Lean first: no polling where Windows offers an event, nothing running while nothing is on screen, no new
  dependency without a reason written in the PR. No Electron, no bundled browser, no background services.
- Every new Rust → UI payload gets a type in `src/lib/types.ts` (serde `camelCase`).
- New settings: add to `Settings` and `DEFAULTS`; if Rust needs it, read it with a default via `Config`.
- UI copy: sentence case, plain verbs, no all-caps labels; errors say what happened and what to do.
- Do not read or copy TwoCode's code; features from it are re-implemented from its public description only.
- Keep `mock.ts` in step with new commands so the browser preview keeps working.
