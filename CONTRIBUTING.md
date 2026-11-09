# Contributing to Holm

Thanks for wanting to help. Holm is open for development: bug reports, feature requests, fixes, new themes,
translations of the docs, and ports to macOS and Linux are all welcome.

## Ways to help

- **Report a bug**: [open a bug report](https://github.com/IndraTensei/holm/issues/new?template=bug_report.yml).
  Say which Holm and Windows versions you have and what you did; a screenshot or a short clip helps a lot.
- **Ask for a feature**: [open a feature request](https://github.com/IndraTensei/holm/issues/new?template=feature_request.yml).
  Say what you'd like to see and why; the why matters more than the how.
- **Ask a question or share an idea**: use [Discussions](https://github.com/IndraTensei/holm/discussions).
- **Port Holm to macOS or Linux**: see [#10 macOS](https://github.com/IndraTensei/holm/issues/10) and
  [#11 Linux](https://github.com/IndraTensei/holm/issues/11). Comment there with the part you're taking.
- **Pick up an issue**: look for [good first issue](https://github.com/IndraTensei/holm/labels/good%20first%20issue)
  and [help wanted](https://github.com/IndraTensei/holm/labels/help%20wanted). Comment before you start so
  nobody else does the same work.

For anything bigger than a small fix, open an issue first so we can agree on the approach before you spend
time on code.

## Setting up

You need Windows 10 or 11 for the full app. You don't for UI work (see below).

1. [Node.js](https://nodejs.org) 22 or later.
2. [Rust](https://rustup.rs) stable, with the MSVC toolchain and the
   [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   (the "Desktop development with C++" workload).
3. The WebView2 runtime (already on Windows 11).

```sh
git clone https://github.com/IndraTensei/holm.git
cd holm
npm ci
npx tauri dev       # the real app, with hot reload for the UI
```

### UI work on any system

The island and the settings window run in a browser with sample data, no Rust and no Windows needed:

```sh
npm ci
npm run dev         # http://localhost:1420
```

Add a scene to the URL to see a state: `?open=music`, `?open=agents`, `?peek=done`, `?approval=1`,
`?timer=1`, `?game=Valorant`, `?front=chrome&near=1`, `?style=pill`, `?theme=<id>`, `?title=lyrics`,
`#settings&section=look`. The full list is in [CLAUDE.md](CLAUDE.md). Sample data lives in `src/lib/mock.ts`.

## Before you open a pull request

Run these and make sure they pass. CI runs the same on every pull request:

```sh
npm run check                                          # 0 errors and 0 warnings
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Then:

- Keep one change per pull request, and fill in the template.
- Add screenshots or a short clip for anything you can see.
- Say in the pull request how you tested it, including on which Windows version.
- Update the README or `docs/` when you change what Holm does or how it's built.

## How the code is laid out

- `src-tauri/src/*.rs`: one Rust module per integration (media, system, agents, island window...). Each emits
  events and exposes commands registered in `lib.rs`.
- `src/lib/stores/*.svelte.ts`: one store per integration (Svelte 5 runes); `settings.svelte.ts` holds the
  defaults.
- `src/lib/views`: island states. `src/lib/pages`: the expanded pages. `src/lib/settings`: the settings window.
- `src/lib/themes.ts`: themes, fonts and colours. `src/styles`: base CSS and bundled font faces.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) explains how it fits together.

## House rules

Holm's whole point is to stay small and quiet, so a few rules are firm:

- **Lean first.** No polling where the system offers an event, nothing running while nothing is on screen,
  no looping animation at 60 fps (loops step on the shared 8 fps beat). Measure CPU and memory before and after
  anything that runs all the time.
- **No new dependency without a reason** written in the pull request. No Electron, no bundled browser, no
  background services.
- **Types for every payload.** Every new Rust → UI event or command result gets a type in `src/lib/types.ts`
  (serde `camelCase`).
- **Settings** go in `Settings` and `DEFAULTS` in `settings.svelte.ts`; Rust reads them with a default
  through `Config`.
- **Keep the preview working.** New commands get a mock in `src/lib/mock.ts`.
- **Writing in the UI**: sentence case, plain verbs, no all-caps labels. Errors say what happened and what to
  do next.
- **Privacy**: nothing leaves the PC unless the user turned that feature on, and the README's Privacy section
  says what is sent.

## Commits and releases

- Write commit messages in the imperative, with a short first line ("Fix the lyric line fading its last
  word").
- Releases are tagged `vX.Y.Z` from `main`. The tag builds the installer and the portable exe and publishes a
  release with notes from `docs/releases/vX.Y.Z.md`. [CHANGELOG.md](CHANGELOG.md) lists every release.

## Code of conduct

Everyone taking part is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security problems go
through [SECURITY.md](SECURITY.md), not public issues.

By contributing, you agree that your contribution is licensed under the [MIT License](LICENSE).
