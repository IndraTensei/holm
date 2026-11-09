# Changelog

Every release of Holm, newest first. The format follows [Keep a Changelog](https://keepachangelog.com), and
versions follow [Semantic Versioning](https://semver.org). Downloads are on the
[releases page](https://github.com/IndraTensei/holm/releases).

## [Unreleased]

### Added

- Contributor guide, code of conduct, security policy, issue and pull request templates.
- Dependency updates arrive as one combined pull request a quarter.
- Tracking issues for the macOS (#10) and Linux (#11) versions.

### Changed

- The release build keeps the NSIS tools it downloads and retries bundling once, so one network hiccup no
  longer fails a release.

## [0.1.2] - 2026-11-09

### Added

- **Make way**, the new default for Settings → General → Over a maximized window: the full island over a
  maximized app, dropping to a thin line while the pointer is near it and coming back when it moves off.
- A link to [holm.indratensei.me](https://holm.indratensei.me) in Settings → About.
- A Supported systems table in the README.

## [0.1.1] - 2026-11-09

### Fixed

- The last word of a lyric line in the resting island no longer fades out when the line fits.
- A lyric line too long for the island slides once to its end instead of starting a slow slide it never
  finished.

### Changed

- Lyrics in the resting island move on with one timer per line instead of a 4 Hz clock, and keep one width for
  the whole song instead of resizing on every line.
- Settings → About explains the "Microsoft Edge WebView2" processes in Task Manager.

## [0.1.0] - 2026-11-09

The first release.

### Added

- Music from Spotify, browsers and any player in the Windows media card: cover, synced lyrics, seek, shuffle,
  repeat, the playing app's own volume and an accent colour from the album art. Likes and exact progress with
  Pear Desktop.
- Claude Code: live activity per session, Allow or Deny from the island, a chime when a task finishes, and
  plan limits as a ring.
- Harnesses: the AI coding agents on the PC, which are running and what each used today.
- Games: an intro when a game starts, out of the way while playing, missed agent news afterwards.
- Shelf, clipboard history, quick note, focus timer, weather, Wi-Fi and Bluetooth toggles, system readings.
- Notch, Pill and Line shapes, 15 themes plus your own, 13 typefaces, three motion styles.
- An installer with an options page (start with Windows, Start menu and desktop shortcuts) and a portable exe.

[Unreleased]: https://github.com/IndraTensei/holm/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/IndraTensei/holm/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/IndraTensei/holm/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/IndraTensei/holm/releases/tag/v0.1.0
