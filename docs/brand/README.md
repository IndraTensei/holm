# Holm brand

A holm is a small island in a river. The mark is a solid H whose two counters are the island's notch shape:
the island hanging from the top edge, and its reflection in the water below.

## Files

| File | Use |
| --- | --- |
| `holm-logo.svg` / `holm-logo-dark.svg` | Horizontal logo for light / dark backgrounds (README, site header) |
| `holm-logo-stacked.svg` / `holm-logo-stacked-dark.svg` | Stacked logo for square spaces |
| `holm-logo-ink.svg` / `holm-logo-white.svg` | One-colour logo (print, single-colour use) |
| `holm-mark.svg`, `-ink`, `-white` | The mark alone |
| `holm-mark-small.svg` | The mark redrawn on a 32-unit pixel grid: use it at 32 px and below (tray, favicon) |
| `holm-app-icon.svg` | App icon master (1024), with the gentle top-to-bottom gradient |
| `holm-wordmark.svg`, `-white` | The name alone |

The app icons in `src-tauri/icons` come from `holm-app-icon.svg` (40 px and up) and `holm-mark-small.svg`
(32 px and below).

## Colour

| Name | Hex | Use |
| --- | --- | --- |
| Periwinkle | `#6F7FF7` | The mark. 3.5:1 on white, 4.7:1 on the Windows dark taskbar |
| Mist | `#9FB3FF` | Accent on dark (the island's default accent) |
| Ink | `#0B0B0F` | Wordmark on light, the island itself |
| Paper | `#F5F5F7` | Light backgrounds |

The mark may also be Ink or white when only one colour is possible. Never another colour, never a photo fill.

## Type

The wordmark is Bricolage Grotesque ExtraBold (800) with -8/200 tracking, outlined, so no font is needed to use the files.
Bricolage Grotesque is under the SIL Open Font License, which allows logo use. UI and captions use Geist.

## Clear space and size

- Keep clear space around the logo equal to the H's crossbar height (a quarter of the mark's height).
- Smallest sizes: the mark at 16 px (use the small file), the horizontal logo at 80 px wide.
- In the horizontal logo the name sits to the right of the mark, its height about 0.6× the mark's. Do not resize
  the two parts independently.

## Do not

- Stretch, rotate, outline or add shadows or glows to the mark.
- Put the Periwinkle mark on mid-blue or purple backgrounds; use the white or Ink mark there.
- Rebuild the wordmark in another font, or set "Holm" next to the mark in live text as the logo.
