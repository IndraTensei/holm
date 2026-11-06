"""Screenshots of island states in headless Chrome, over a desktop-like backdrop.

Usage: python3 scripts/shots.py [base_url] [out_dir]
The app must be served (npx vite preview --port 4173). Only for design review.
"""
import sys
from pathlib import Path

from playwright.sync_api import sync_playwright

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:4173/"
OUT = Path(sys.argv[2] if len(sys.argv) > 2 else "shots")
OUT.mkdir(parents=True, exist_ok=True)

SCENES = {
    "01-idle": "?music=off&agents=off",
    "02-music": "?agents=off",
    "03-music-title": "?agents=off&title=lyrics",
    "04-music-agent-bubble": "",
    "05-approval": "?approval=1",
    "06-peek-done": "?peek=done&agents=off",
    "07-peek-track": "?peek=track&agents=off",
    "08-peek-volume": "?peek=volume&agents=off",
    "09-open-music": "?open=music&agents=off",
    "10-open-agents": "?open=agents",
    "11-open-home": "?open=home",
    "12-open-system": "?open=system",
    "13-open-shelf": "?open=shelf",
    "14-open-focus": "?open=focus&agents=off&music=off",
    "15-pill-nord": "?style=pill&theme=nord&agents=off",
    "16-open-music-porcelain": "?open=music&theme=porcelain&agents=off",
    "17-open-home-arena": "?open=home&theme=arena",
    "18-open-agents-sakura": "?open=agents&theme=sakura",
    "19-drop": "?drop=1",
    "20-timer": "?timer=1&music=off&agents=off",
}

BACKDROP = """
html { background: radial-gradient(1200px 600px at 20% 10%, #3a5a8c 0%, transparent 60%),
  radial-gradient(900px 500px at 90% 0%, #7a4a7f 0%, transparent 55%), linear-gradient(160deg, #1d2b44, #3b2a4a 60%, #1a2333) !important; }
html.light { background: linear-gradient(160deg, #cfe0f5, #f3e6ef 60%, #e6eef7) !important; }
"""

with sync_playwright() as p:
    browser = p.chromium.launch(executable_path="/usr/bin/google-chrome", args=["--no-sandbox"])
    page = browser.new_page(viewport={"width": 760, "height": 560}, device_scale_factor=2)
    for name, query in SCENES.items():
        page.goto(BASE + query)
        page.add_style_tag(content=BACKDROP)
        if "porcelain" in query or "sakura" in query:
            page.evaluate("document.documentElement.classList.add('light')")
        page.wait_for_timeout(2600)
        page.screenshot(path=str(OUT / f"{name}.png"))
        print("shot", name)
    # Settings window
    page = browser.new_page(viewport={"width": 940, "height": 680}, device_scale_factor=1.5)
    for section in ["general", "look", "music", "agents"]:
        page.goto(BASE + f"?section={section}#settings")
        page.wait_for_timeout(1200)
        page.screenshot(path=str(OUT / f"settings-{section}.png"))
        print("shot settings", section)
    browser.close()
