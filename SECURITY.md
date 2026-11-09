# Security policy

## Supported versions

Security fixes go into the latest release. Please update to it before reporting.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |

## Reporting a vulnerability

Please don't open a public issue for a security problem. Report it privately through
[GitHub's private vulnerability reporting](https://github.com/IndraTensei/holm/security/advisories/new),
or by email to ashishthomas012@gmail.com.

Include what you found, how to reproduce it, and what an attacker could do with it. You'll get a reply within
a week. Once a fix is out, the advisory is published with credit to you, unless you'd rather stay anonymous.

## What's in scope

Holm runs on your PC with your user's rights. These parts matter most:

- **The local hook server** on `127.0.0.1:47821`, which Claude Code calls to report activity and ask for
  permission decisions. It accepts requests only with the token Holm writes into `~/.claude/settings.json`.
- **Permission decisions**: anything that could make Holm answer Allow without the user choosing it.
- **The installer and autostart entry**: what they write to disk and the registry.
- **Files Holm reads**: Claude Code and Codex sign-in files (only to ask for plan usage, when turned on),
  harness logs, the clipboard.
- **Network requests**: plan usage (Anthropic, OpenAI), lyrics (LRCLIB), weather (Open-Meteo), Pear's local
  API, and the update check (GitHub's latest-release API).
- **Updates**: the installer is downloaded only from this repository's release assets over HTTPS, checked for
  size, type and the SHA-256 GitHub publishes for the file, and run in passive mode. Anything that could make Holm download or run a different file is in
  scope.

Holm isn't code-signed yet, so Windows SmartScreen warns on first run; that by itself isn't a vulnerability.
