"""Record the promo clips: the browser preview, driven by scripted events, captured frame by frame.

Usage: python scripts/film.py [base_url] [out_dir] [clip ...]
Needs the preview (npm run dev, :1420), Playwright and ffmpeg on PATH. Each clip becomes
<out_dir>/<clip>.mp4 at 60 fps, from Chromium's screencast frames and their own timestamps.
Headless screencast tops out near 30 fps, so the page runs at half speed while it is filmed
(animations through the DevTools Animation domain, timers through Playwright's clock) and the clip
is played back twice as fast.
Not part of the app: mock data only (src/lib/mock.ts, `?film=1`).
"""
import asyncio
import base64
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

from playwright.async_api import async_playwright

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:1420/"
OUT = Path(sys.argv[2] if len(sys.argv) > 2 else "film")
ONLY = set(sys.argv[3:])
W, H = 960, 540  # CSS px; each scene is filmed at 1920 px wide
SLOW = 3  # filmed at 1/SLOW speed, played back at SLOW×
TICK = 1000 / 60  # page time per step, ms
PAGE_MS = [0.0, 0]  # page time so far: exact, and as whole milliseconds handed to the clock

# A desktop-like backdrop, plus a drawn pointer (the screencast has no cursor of its own).
STAGE = """
html { background: radial-gradient(1100px 620px at 18% 12%, #35558a 0%, transparent 60%),
  radial-gradient(900px 520px at 88% 4%, #7b4a82 0%, transparent 58%),
  radial-gradient(800px 600px at 60% 110%, #1f6f78 0%, transparent 60%),
  linear-gradient(160deg, #18233a, #33264a 60%, #141c2c) !important; }
#film-cursor { position: fixed; left: 0; top: 0; width: 22px; height: 22px; z-index: 2147483647;
  pointer-events: none; transform: translate(-100px, -100px); transition: opacity .2s; }
"""
CURSOR = """
(() => {
  const c = document.createElement('div');
  c.id = 'film-cursor';
  c.innerHTML = `<svg viewBox="0 0 24 24" width="22" height="22"><path d="M4 2.5v17.2l4.6-4.3 2.9 6.6 2.7-1.2-2.9-6.5h6.3z"
    fill="#fff" stroke="#111" stroke-width="1.3" stroke-linejoin="round"/></svg>`;
  document.body.appendChild(c);
  addEventListener('pointermove', (e) => (c.style.transform = `translate(${e.clientX - 3}px, ${e.clientY - 2}px)`));
})();
"""

ART = {
    "a": ("#ff7a59", "#6a3df0", "#ffd166"),
    "b": ("#0fb9b1", "#2d3a8c", "#f7d794"),
}


def art(key: str) -> str:
    a, b, c = ART[key]
    svg = (
        f"<svg xmlns='http://www.w3.org/2000/svg' width='120' height='120'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'>"
        f"<stop offset='0' stop-color='{a}'/><stop offset='1' stop-color='{b}'/></linearGradient></defs>"
        f"<rect width='120' height='120' fill='url(#g)'/><circle cx='84' cy='38' r='22' fill='{c}' opacity='0.85'/>"
        f"<rect x='14' y='78' width='70' height='8' rx='4' fill='white' opacity='0.5'/>"
        f"<rect x='14' y='92' width='44' height='8' rx='4' fill='white' opacity='0.3'/></svg>"
    )
    return "data:image/svg+xml;base64," + base64.b64encode(svg.encode()).decode()


# JS helpers run in the page: build payloads with the page's own clock.
HELPERS = """
window.F = {
  emit: (e, p) => window.__film.emit(e, p),
  track(title, artist, art, position) {
    const now = Date.now();
    F.emit('media://state', { current: 'Spotify.exe', sessions: [{ id: 'Spotify.exe', app: 'Spotify', title, artist,
      album: 'Backwater Radio', art, status: 'playing', position, duration: 214, updatedAt: now, canPlayPause: true,
      canNext: true, canPrev: true, canSeek: true, shuffle: false, repeat: 'none' }] });
  },
  session(status, extra = {}) {
    const now = Date.now();
    return { id: 's1', agent: 'claude', project: 'tensei-island', cwd: 'D:\\\\code\\\\tensei-island', status,
      activity: status === 'working' ? { verb: 'Editing', target: 'island.rs', tool: 'Edit', kind: 'edit' } : null,
      prompt: 'Make the hover area follow the island while it animates', message: '', turnStartedAt: now - 96000,
      updatedAt: now, tools: 12, edited: ['island.rs', 'IslandApp.svelte'], subagents: 0,
      log: [{ at: now - 4000, kind: 'edit', text: 'Editing island.rs' }, { at: now - 21000, kind: 'run', text: 'Running cargo test' },
        { at: now - 47000, kind: 'read', text: 'Reading IslandApp.svelte' }, { at: now - 96000, kind: 'prompt', text: 'Make the hover area follow the island while it animates' }],
      ...extra };
  },
  agents(status, approvals = []) { F.emit('agents://state', { sessions: [F.session(status)], approvals }); },
  approval() {
    const now = Date.now();
    const a = { id: 'a1', sessionId: 's1', project: 'tensei-island', tool: 'Bash', title: 'Running cargo build --release',
      detail: 'cargo build --release', createdAt: now, expiresAt: now + 60000 };
    F.agents('waiting', [a]);
    F.emit('agents://approval', a);
  },
  done() {
    F.emit('agents://state', { sessions: [F.session('done', { message: 'The hover area now follows the island while it animates. Tests pass.' })], approvals: [] });
    F.emit('agents://done', { sessionId: 's1', project: 'tensei-island', message: 'The hover area now follows the island while it animates. Tests pass.',
      durationMs: 154000, tools: 18, edited: ['island.rs', 'IslandApp.svelte', 'fit.svelte.ts'] });
  },
  theme(id) { F.emit('settings://changed', { theme: id }); },
};
"""


class Clip:
    def __init__(self, page, cdp):
        self.page, self.cdp, self.frames = page, cdp, []

    async def start(self):
        self.cdp.on("Page.screencastFrame", self._frame)
        await self.cdp.send("Page.startScreencast", {"format": "jpeg", "quality": 95, "everyNthFrame": 1})

    def _frame(self, p):
        self.frames.append((p["metadata"]["timestamp"], p["data"]))
        asyncio.ensure_future(self.cdp.send("Page.screencastFrameAck", {"sessionId": p["sessionId"]}))

    async def stop(self, name: str):
        end = time.time()
        await self.cdp.send("Page.stopScreencast")
        work = OUT / name
        shutil.rmtree(work, ignore_errors=True)
        work.mkdir(parents=True)
        lines = ["ffconcat version 1.0"]
        times = [t for t, _ in self.frames] + [max(end, self.frames[-1][0] + 0.1)]
        for i, (t, data) in enumerate(self.frames):
            f = work / f"{i:05d}.jpg"
            f.write_bytes(base64.b64decode(data))
            lines += [f"file '{f.name}'", f"duration {max(times[i + 1] - t, 0.001):.4f}"]
        lines.append(f"file '{work.joinpath(f'{len(self.frames) - 1:05d}.jpg').name}'")
        (work / "list.txt").write_text("\n".join(lines))
        subprocess.run(
            ["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(work / "list.txt"),
             "-vf", f"setpts=PTS/{SLOW},fps=60,scale=1920:1080:flags=lanczos,format=yuv420p", "-c:v", "libx264", "-preset", "slow",
             "-crf", "12", str(OUT / f"{name}.mp4")],
            check=True,
        )
        shutil.rmtree(work)
        gaps = sorted(times[i + 1] - times[i] for i in range(len(self.frames)))
        busy = [g for g in gaps if g < 0.1]
        rate = len(busy) / sum(busy) if busy else 0
        print(f"{name}: {len(self.frames)} frames, {(times[-1] - times[0]) / SLOW:.1f} s, {rate * SLOW:.0f} fps while moving")


async def island_center(page):
    box = await page.locator('[aria-label="Tensei Island"]').bounding_box()
    return box["x"] + box["width"] / 2, box["y"] + box["height"] / 2


POS = {"x": W - 40.0, "y": H - 40.0}


async def glide(page, x, y, secs=0.7):
    """Move the pointer with an ease-in-out, one step per page frame."""
    x0, y0 = POS["x"], POS["y"]
    n = max(1, round(secs * 60))
    for i in range(1, n + 1):
        t = i / n
        e = 4 * t * t * t if t < 0.5 else 1 - (-2 * t + 2) ** 3 / 2
        await page.mouse.move(x0 + (x - x0) * e, y0 + (y - y0) * e)
        await wait(page, 1 / 60)
    POS["x"], POS["y"] = x, y


async def hover_in(page):
    x, y = await island_center(page)
    await glide(page, x + 12, y + 4, 0.75)


async def leave(page):
    x, y = await island_center(page)
    await glide(page, x + 330, y + 430, 0.6)


async def scene(browser, name, query, act, viewport=(W, H)):
    if ONLY and name not in ONLY:
        return
    ctx = await browser.new_context(viewport={"width": viewport[0], "height": viewport[1]}, device_scale_factor=1920 / viewport[0])
    page = await ctx.new_page()
    # A fixed evening on today's date, so the Home clock and countdowns read naturally.
    await page.clock.install(time=datetime.now().replace(hour=19, minute=42, second=0, microsecond=0))
    await page.goto(BASE + query)
    await page.add_style_tag(content=STAGE)
    await page.evaluate(CURSOR)
    if "film" in query:
        await page.evaluate(HELPERS)
    POS["x"], POS["y"] = viewport[0] - 40.0, viewport[1] - 40.0
    await page.mouse.move(POS["x"], POS["y"])
    await page.wait_for_timeout(1800)  # fonts, first state, settle
    cdp = await ctx.new_cdp_session(page)
    await page.clock.pause_at((await page.evaluate("Date.now()") + 20) / 1000)  # seconds in Python
    await cdp.send("Animation.enable")
    await cdp.send("Animation.setPlaybackRate", {"playbackRate": 1 / SLOW})
    clip = Clip(page, cdp)
    await clip.start()
    await act(page)
    await clip.stop(name)
    await ctx.close()


async def wait(page, s):
    """Let `s` seconds of page time pass. The page clock follows real time / SLOW, the same rate
    as the slowed animations, however long each clock step takes to run."""
    loop = asyncio.get_running_loop()
    goal = PAGE_MS[0] + s * 1000
    last = loop.time()
    while PAGE_MS[0] < goal:
        await asyncio.sleep(max(0.0, SLOW * TICK / 1000 - (loop.time() - last)))
        now = loop.time()
        PAGE_MS[0] = min(goal, PAGE_MS[0] + (now - last) * 1000 / SLOW)
        last = now
        step = int(PAGE_MS[0]) - PAGE_MS[1]  # whole milliseconds for the clock
        if step > 0:
            PAGE_MS[1] += step
            await page.clock.run_for(step)


async def main():
    OUT.mkdir(parents=True, exist_ok=True)
    a, b = art("a"), art("b")

    async def idle_to_music(p):
        await wait(p, 1.6)
        await p.evaluate(f"F.track('Lanterns Over Kochi', 'The Monsoon Office', '{a}', 64)")
        await wait(p, 3.2)
        await p.evaluate(f"F.track('Night Ferry', 'Harbour Lights', '{b}', 0)")
        await wait(p, 4.2)

    async def music_open(p):
        await p.evaluate(f"F.track('Lanterns Over Kochi', 'The Monsoon Office', '{a}', 71)")
        await wait(p, 1.2)
        await hover_in(p)
        await wait(p, 5.5)
        await leave(p)
        await wait(p, 1.5)

    async def agent(p):
        await wait(p, 1.0)
        await p.evaluate("F.agents('working')")
        await wait(p, 3.4)
        await hover_in(p)
        await wait(p, 4.5)
        await leave(p)
        await wait(p, 2.2)

    async def approval(p):
        await p.evaluate("F.agents('working')")
        await wait(p, 1.6)
        await p.evaluate("F.approval()")
        await wait(p, 2.6)
        box = await p.locator("button.allow").bounding_box()
        bx, by = box["x"] + box["width"] / 2, box["y"] + box["height"] / 2
        await glide(p, bx, by, 0.9)
        await wait(p, 0.35)
        await p.mouse.down()
        await wait(p, 0.08)
        await p.mouse.up()
        await p.evaluate("F.agents('working')")
        await wait(p, 0.2)
        await glide(p, bx + 320, by + 380, 0.7)
        await wait(p, 2.2)
        await p.evaluate("F.done()")
        await wait(p, 5.0)

    async def home(p):
        await wait(p, 5.0)

    async def system(p):
        await wait(p, 4.0)

    async def themes(p):
        await wait(p, 1.0)
        for t in ["sakura", "synthwave", "nord", "porcelain", "catppuccin", "arena", "obsidian"]:
            await p.evaluate(f"F.theme('{t}')")
            await wait(p, 1.3)

    async def game(p):
        await wait(p, 1.2)
        await p.evaluate("F.emit('game://launch', { name: 'Neon Drift', exe: 'neondrift.exe' })")
        await wait(p, 5.0)

    async def harnesses(p):
        await wait(p, 2.5)
        await glide(p, 700, 420, 0.8)
        for _ in range(40):
            await p.mouse.wheel(0, 11)
            await wait(p, 1 / 30)
        await wait(p, 2.0)

    async with async_playwright() as pw:
        browser = await pw.chromium.launch(channel="msedge", headless=True)
        await scene(browser, "01-music-start", "?agents=off&music=off&film=1", idle_to_music)
        await scene(browser, "02-music-open", "?agents=off&music=off&film=1&title=lyrics", music_open)
        await scene(browser, "03-agent", "?music=off&agents=off&film=1", agent)
        await scene(browser, "04-approval", "?music=off&agents=off&film=1", approval)
        await scene(browser, "05-home", "?open=home&film=1", home)
        await scene(browser, "06-system", "?open=system&film=1", system)
        await scene(browser, "07-themes", "?agents=off&open=music&film=1", themes)
        await scene(browser, "08-game", "?music=off&agents=off&film=1", game)
        await scene(browser, "09-harnesses", "?section=tools#settings", harnesses, viewport=(1120, 630))
        await browser.close()


asyncio.run(main())
