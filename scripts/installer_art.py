"""Draws the installer's sidebar (welcome/finish pages) and header bitmaps from the brand SVGs.

python scripts/installer_art.py   (needs Pillow and cairosvg)
Writes src-tauri/windows/sidebar.bmp (164x314) and header.bmp (150x57), the sizes NSIS expects.
"""

import io
from pathlib import Path

import cairosvg
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "docs" / "brand"
OUT = ROOT / "src-tauri" / "windows"
TOP, BOTTOM = (0x87, 0x97, 0xFF), (0x5B, 0x6A, 0xEE)  # the app icon's gradient


def svg(name: str, width: int) -> Image.Image:
    png = cairosvg.svg2png(url=str(BRAND / name), output_width=width)
    return Image.open(io.BytesIO(png)).convert("RGBA")


def sidebar() -> Image.Image:
    w, h = 164, 314
    img = Image.new("RGB", (w, h))
    draw = ImageDraw.Draw(img)
    for y in range(h):
        t = y / (h - 1)
        draw.line([(0, y), (w, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(TOP, BOTTOM)))
    mark = svg("holm-mark-white.svg", 92)  # 256 viewBox with 24 margin: the block is ~72 px
    img.paste(mark, ((w - mark.width) // 2, 74), mark)
    word = svg("holm-wordmark-white.svg", 70)
    img.paste(word, ((w - word.width) // 2, 74 + mark.height + 6), word)
    return img


def header() -> Image.Image:
    w, h = 150, 57
    img = Image.new("RGB", (w, h), (255, 255, 255))
    logo = svg("holm-logo.svg", 104)
    img.paste(logo, ((w - logo.width) // 2, (h - logo.height) // 2), logo)
    return img


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    sidebar().save(OUT / "sidebar.bmp")
    header().save(OUT / "header.bmp")
    print("wrote", OUT / "sidebar.bmp", OUT / "header.bmp")
