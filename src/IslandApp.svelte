<script lang="ts">
  // The island window: one shape that morphs between resting, peek, approval, drop and open.
  import { onMount, tick } from 'svelte';
  import { FolderDown } from '@lucide/svelte';
  import CompactView from './lib/views/CompactView.svelte';
  import PeekView from './lib/views/PeekView.svelte';
  import ApprovalCard from './lib/views/ApprovalCard.svelte';
  import Expanded from './lib/views/Expanded.svelte';
  import Art from './lib/ui/Art.svelte';
  import Ring from './lib/ui/Ring.svelte';
  import AgentGlyph from './lib/ui/AgentGlyph.svelte';
  import { settings, type PageId } from './lib/stores/settings.svelte';
  import { media } from './lib/stores/media.svelte';
  import { agents } from './lib/stores/agents.svelte';
  import { system } from './lib/stores/system.svelte';
  import { lyrics } from './lib/stores/lyrics.svelte';
  import { activity } from './lib/stores/activity.svelte';
  import { focus, shelf, note, weather } from './lib/stores/widgets.svelte';
  import { clock } from './lib/stores/clock.svelte';
  import { beat } from './lib/stores/beat.svelte';
  import { fit } from './lib/stores/fit.svelte';
  import { inTauri, on, send, call, focusWindow, onFileDrop, registerHotkeys } from './lib/bridge';
  import { findTheme, themeVars, applyVars, accentFromArt } from './lib/themes';
  import { artColor } from './lib/artColor';
  import { motionFor } from './lib/motion';
  import { chime } from './lib/sound';
  import type { Front, Game, GameEnded, IslandState, Live } from './lib/types';

  type Mode = 'compact' | 'peek' | 'approval' | 'drop' | 'expanded';

  let expanded = $state(false);
  let openedBy = $state<'hover' | 'click' | 'hotkey'>('hover');
  let pinned = $state(false);
  let page = $state<PageId>('home');
  let hover = $state(false);
  let dropping = $state(false);
  let size = $state({ w: 176, h: 34 });
  let islandEl = $state<HTMLDivElement>();
  let bubbleEl = $state<HTMLButtonElement>();
  let artAccent = $state<string | null>(null);

  const s = $derived(settings.v);
  const motion = $derived(motionFor(s.motion));
  const compactH = 34;

  const approval = $derived(s.autoOpenApprovals ? (agents.approvals[0] ?? null) : null);

  const live = $derived.by((): Live => {
    const musicLive = media.now !== null && media.live(clock.now);
    if (musicLive && media.now?.playing) return 'music';
    if (agents.focus) return 'agent';
    if (focus.phase !== 'idle') return 'timer';
    if (musicLive) return 'music';
    return 'idle';
  });

  const secondary = $derived.by((): Live | null => {
    const options: Live[] = [];
    if (agents.focus) options.push('agent');
    if (focus.phase !== 'idle') options.push('timer');
    if (media.now?.playing) options.push('music');
    return options.find((o) => o !== live) ?? null;
  });

  const mode = $derived.by((): Mode => {
    if (dropping) return 'drop';
    if (expanded) return 'expanded';
    if (approval) return 'approval';
    if (activity.peek) return 'peek';
    return 'compact';
  });

  // Rest as a thin line after a while with nothing going on; anything live, a peek or the
  // pointer brings the island back.
  let tucked = $state(false);
  $effect(() => {
    if (!(s.tuckWhenIdle && live === 'idle' && mode === 'compact' && !hover)) {
      tucked = false;
      return;
    }
    const timer = setTimeout(() => (tucked = true), Math.max(3, s.tuckAfter) * 1000);
    return () => clearTimeout(timer);
  });

  const style = $derived(s.style === 'line' || tucked || fit.line ? 'line' : s.style);

  // WebView2 in a transparent window can keep showing an old frame outside the area it last
  // redrew (seen as a faded copy of the open island after it shrank to a line). Once a change
  // has settled, repaint the whole surface once.
  $effect(() => {
    void viewKey;
    void style;
    const timer = setTimeout(() => {
      const root = document.documentElement;
      root.style.opacity = '0.999';
      requestAnimationFrame(() => requestAnimationFrame(() => (root.style.opacity = '')));
    }, motion.duration + 150);
    return () => clearTimeout(timer);
  });

  const lineNow = $derived(style === 'line' && mode === 'compact');
  const notch = $derived(style === 'notch');

  const viewKey = $derived(
    mode === 'expanded'
      ? `x:${page}`
      : mode === 'peek'
        ? `p:${activity.peek?.id}`
        : mode === 'approval'
          ? `a:${approval?.id}`
          : mode === 'drop'
            ? 'd'
            : `c:${live}:${lineNow}`,
  );

  const radius = $derived.by(() => {
    if (lineNow) return 6;
    if (mode === 'compact') return size.h / 2;
    if (mode === 'expanded') return 30;
    return 24;
  });

  const showBubble = $derived(mode === 'compact' && !lineNow && secondary !== null);

  // ------------------------------------------------------------ open and close

  function contextualPage(): PageId {
    const peek = activity.peek?.kind;
    if (peek === 'done' || peek === 'attention') return 'agents';
    if (peek === 'track' || peek === 'volume') return 'music';
    if (peek === 'timer') return 'focus';
    if (peek === 'shelf') return 'shelf';
    if (live === 'music') return 'music';
    if (live === 'agent') return 'agents';
    if (live === 'timer') return 'focus';
    return page;
  }

  function open(by: 'hover' | 'click' | 'hotkey', target?: PageId) {
    if (!expanded) page = target ?? contextualPage();
    else if (target) page = target;
    if (!s.pages.includes(page)) page = s.pages[0] ?? 'home';
    openedBy = by;
    if (!expanded) fit.opened(by);
    expanded = true;
    activity.dismiss();
  }

  // Brought back with the shortcut over a game, full screen or a hidden island.
  let summoned = false;

  function close() {
    if (expanded) fit.closed();
    expanded = false;
    pinned = false;
    if (inTauri && document.hasFocus()) send('island_release_focus');
    if (summoned) {
      summoned = false;
      send('island_summon', { on: false });
    }
  }

  async function toggle() {
    if (expanded) return close();
    summoned = true;
    await call('island_summon', { on: true }).catch(() => {});
    open('hotkey');
    void focusWindow();
  }

  // While a game is in front (the island is hidden) or Windows' Do not disturb is on, agent
  // news waits here without chimes, and shows as one card afterwards.
  let inGame: Game | null = null;
  let quiet = false;
  let heldBy: 'game' | 'quiet' = 'game';
  let missed: { project: string; text: string }[] = [];

  function holding(): boolean {
    if (inGame) heldBy = 'game';
    else if (quiet) heldBy = 'quiet';
    return Boolean(inGame || quiet);
  }

  let dndOn = $state(false);
  $effect(() => setQuiet(dndOn && s.respectDnd));

  function setQuiet(on: boolean) {
    quiet = on;
    activity.quiet = quiet;
    if (!quiet && !inGame) flushMissed();
  }

  function flushMissed() {
    if (!missed.length) return;
    const title = heldBy === 'game' ? 'While you were playing' : 'While Do not disturb was on';
    const data = { title, items: missed.slice(-4), count: missed.length };
    missed = [];
    // Let the play-time card have its moment first.
    if (activity.peek?.kind === 'gameover') setTimeout(() => activity.show('missed', data, 8000), 5000);
    else activity.show('missed', data, 8000);
  }

  let hidden = false;
  function toggleHidden() {
    send('island_set_hidden', { hidden: !hidden });
  }

  // Where the pointer last moved inside this window. Opening on hover needs it over the island
  // itself (or the gap above it, or the top edge over a line), not merely inside the hover area.
  let pointer: { x: number; y: number } | null = null;

  function overIsland(): boolean {
    if (!pointer || !islandEl) return false;
    if (edgeOnly) return pointer.y <= 6;
    const pad = 6;
    return [islandEl, bubbleEl].some((el) => {
      if (!el) return false;
      const r = el.getBoundingClientRect();
      return pointer!.x >= r.left - pad && pointer!.x <= r.right + pad && pointer!.y <= r.bottom + pad;
    });
  }

  let enterTimer: ReturnType<typeof setTimeout> | undefined;
  let leaveTimer: ReturnType<typeof setTimeout> | undefined;

  function setHover(next: boolean) {
    hover = next;
    clearTimeout(enterTimer);
    clearTimeout(leaveTimer);
    if (next) {
      if (!expanded && s.openOn === 'hover' && mode !== 'approval')
        enterTimer = setTimeout(async () => {
          if (!hover) return;
          // Where the pointer really is (no move may have reached this window yet).
          const at = await call<[number, number] | null>('island_cursor').catch(() => null);
          if (at) pointer = { x: at[0], y: at[1] };
          if (!hover || expanded) return;
          if (overIsland()) open('hover');
          else {
            // The hover area Rust holds is bigger than the island (measured mid-animation, say):
            // do not open, and send the real one again.
            lastRect = '';
            reportRect();
          }
        }, s.hoverDelay);
    } else if (expanded && !pinned) {
      leaveTimer = setTimeout(() => {
        const typing = document.activeElement instanceof HTMLTextAreaElement || document.activeElement instanceof HTMLInputElement;
        if (!hover && !pinned && !(openedBy === 'hotkey' && document.hasFocus()) && !typing) close();
      }, s.collapseDelay);
    }
  }

  // ------------------------------------------------------------ measuring and hit area

  function measure(node: HTMLElement, key: string) {
    let current = key;
    const read = () => {
      if (current !== viewKey) return;
      const w = Math.ceil(node.offsetWidth);
      const h = Math.ceil(node.offsetHeight);
      if (w && h && (w !== size.w || h !== size.h)) size = { w, h };
    };
    const ro = new ResizeObserver(read);
    ro.observe(node);
    requestAnimationFrame(read);
    return {
      update(next: string) {
        current = next;
        read();
      },
      destroy() {
        ro.disconnect();
      },
    };
  }

  let lastRect = '';
  // Resting as a line over a maximized app: only the top edge of the screen (a flick of the
  // pointer) opens it, so a browser tab or title bar button under the line can still be clicked.
  const edgeOnly = $derived(lineNow && fit.line && !dropping);

  function reportRect() {
    if (!islandEl) return;
    const scale = s.scale || 1;
    if (edgeOnly) {
      const r = islandEl.getBoundingClientRect();
      const rect = { x: r.left - 40 * scale, y: 0, width: r.width + 80 * scale, height: 1 };
      const key = `edge:${Math.round(rect.x)},${Math.round(rect.width)}`;
      if (key === lastRect) return;
      lastRect = key;
      send('island_set_rect', rect);
      return;
    }
    const r = islandEl.getBoundingClientRect();
    const tw = size.w * scale;
    const th = size.h * scale;
    const tx = (window.innerWidth - tw) / 2;
    let left = Math.min(r.left, tx);
    let right = Math.max(r.right, tx + tw);
    const top = Math.min(r.top, notch ? 0 : r.top);
    let bottom = Math.max(r.bottom, r.top + th);
    if (bubbleEl) {
      const b = bubbleEl.getBoundingClientRect();
      right = Math.max(right, b.right);
      bottom = Math.max(bottom, b.bottom);
    }
    if (notch) {
      left -= 12 * scale;
      right += 12 * scale;
    }
    const rect = { x: left, y: top, width: right - left, height: bottom - top };
    const key = `${Math.round(rect.x)},${Math.round(rect.y)},${Math.round(rect.width)},${Math.round(rect.height)}`;
    if (key === lastRect) return;
    lastRect = key;
    send('island_set_rect', rect);
  }

  $effect(() => {
    size;
    showBubble;
    s.scale;
    style;
    edgeOnly;
    void tick().then(reportRect);
    // Measured at once the island may still be mid-animation (an open island shrinking back), and
    // transitionend does not always come (an interrupted transition): measure again once settled,
    // or a stale, bigger hover area opens the island with the pointer well below it.
    const settled = setTimeout(reportRect, motion.duration + 80);
    return () => clearTimeout(settled);
  });

  // ------------------------------------------------------------ theme

  $effect(() => {
    const theme = findTheme(s.theme, s.customThemes);
    const art = media.now?.art ?? null;
    if (!theme.adaptive || !s.adaptiveAccent || !art) {
      artAccent = null;
      return;
    }
    void artColor(art).then((rgb) => {
      if (media.now?.art === art) artAccent = rgb ? accentFromArt(rgb, theme.mode).accent : null;
    });
  });

  $effect(() => {
    const theme = findTheme(s.theme, s.customThemes);
    applyVars(document.documentElement, themeVars(theme, { fontUi: s.fontUi, fontDisplay: s.fontDisplay, accent: artAccent ?? undefined }));
  });

  // ------------------------------------------------------------ live things that need a fast clock

  $effect(() => (s.compactTitle === 'lyrics' && media.now?.playing ? clock.fast() : undefined));
  $effect(() => (live === 'agent' || live === 'timer' ? clock.second() : undefined));
  $effect(() => {
    media.now?.key;
    s.lyrics;
    void lyrics.load();
  });

  let lastTrack = '';
  $effect(() => {
    const n = media.now;
    if (!n) return;
    if (lastTrack && n.key !== lastTrack && n.playing && s.trackToasts && !expanded) {
      activity.show('track', { title: n.title, artist: n.artist, art: n.art }, 3200);
    }
    lastTrack = n.key;
  });

  $effect(() => {
    const list = [
      { accelerator: s.hotkey, handler: () => void toggle() },
      { accelerator: s.hideHotkey, handler: toggleHidden },
    ];
    if (settings.ready) void registerHotkeys(list);
  });

  // ------------------------------------------------------------ start

  onMount(() => {
    const stops: (() => void)[] = [];
    void (async () => {
      await settings.load();
      await media.start();
      await agents.start({
        done: (d) => {
          if (holding()) return void missed.push({ project: d.project, text: 'is done' });
          activity.show('done', d as unknown as Record<string, unknown>, 7000);
          if (settings.v.agentSounds) chime('done');
        },
        approval: () => {
          if (expanded) page = 'agents';
          if (settings.v.agentSounds && !quiet) chime('ask');
        },
        attention: (a) => {
          if (holding()) return void missed.push({ project: a.project, text: 'needs you' });
          activity.show('attention', a, 6000);
          if (settings.v.agentSounds) chime('ask');
        },
      });
      await system.start(
        (battery, prev) => {
          if (battery?.plugged && prev && !prev.plugged) activity.show('charging', { percent: battery.percent }, 3000);
        },
        (v) => {
          if (settings.v.volumeOsd && !(expanded && page === 'music')) activity.show('volume', { ...v }, 1600);
        },
      );
      await shelf.load();
      await fit.load();
      await note.load();
      weather.start();
      focus.onFinish = (phase) => {
        activity.show('timer', { phase }, 6000);
        chime('timer');
      };
      stops.push(await on<boolean>('island://hover', setHover));
      // Rust hides the window behind WebView2's back (games, full screen, tray): stop looping then.
      stops.push(await on<boolean>('island://shown', (v) => beat.pause(!v)));
      stops.push(await on<boolean>('island://hidden', (v) => (hidden = v)));
      stops.push(await on<boolean>('island://quiet', (v) => (dndOn = v)));
      dndOn = Boolean(await call<boolean>('quiet_get').catch(() => false));
      // Changes are events; the state before this window finished loading is asked for once.
      const state = await call<IslandState>('island_state').catch(() => null);
      if (state) {
        fit.set(state);
        hidden = state.hidden;
        beat.pause(!state.shown);
      }
      stops.push(
        await on<Game>('game://launch', (g) => {
          if (expanded && !pinned) close();
          activity.show('game', { ...g }, 4200);
        }),
      );
      stops.push(
        await on<Game | null>('game://active', (g) => {
          inGame = g && settings.v.smartHide ? g : null;
          if (!inGame && !quiet) flushMissed();
        }),
      );
      stops.push(
        await on<GameEnded>('game://ended', (e) => {
          // Short visits (a crash at start, a quick look) are not worth a recap.
          if (settings.v.gameRecap && e.durationMs >= 60_000) activity.show('gameover', { ...e }, 5000);
        }),
      );
      stops.push(await on('island://toggle', toggle));
      stops.push(await on<Front>('island://maximized', (v) => fit.set(v)));
      stops.push(await on('island://placed', () => ((lastRect = ''), reportRect())));
      stops.push(
        await onFileDrop((e) => {
          if (e.type === 'enter') dropping = true;
          else if (e.type === 'leave') dropping = false;
          else if (e.type === 'drop') {
            dropping = false;
            const count = shelf.add(e.paths);
            activity.show('shelf', { count }, 2500);
          }
        }),
      );
      if (!inTauri) debugScene();
    })();
    const blur = () => {
      if (expanded && openedBy === 'hotkey' && !pinned && !hover) close();
    };
    window.addEventListener('blur', blur);
    return () => {
      window.removeEventListener('blur', blur);
      stops.forEach((stop) => stop());
    };
  });

  /** Browser previews: ?open=music, ?peek=done, ?approval=1, ?drop=1 */
  function debugScene() {
    const q = new URLSearchParams(location.search);
    const openPage = q.get('open') as PageId | null;
    if (openPage) {
      page = openPage;
      expanded = true;
      pinned = true;
    }
    const peek = q.get('peek');
    if (peek === 'done') activity.show('done', { project: 'meetbot', durationMs: 154000, edited: ['db.ts', 'league.ts', 'a.ts'], message: 'Transcripts are now saved per meeting, with a migration and two passing tests.' }, 1e9);
    if (peek === 'track') setTimeout(() => activity.show('track', { title: media.now?.title, artist: media.now?.artist, art: media.now?.art }, 1e9), 300);
    if (peek === 'volume') activity.show('volume', { level: 0.62, muted: false }, 1e9);
    if (peek === 'charging') activity.show('charging', { percent: 78 }, 1e9);
    if (q.get('drop')) dropping = true;
    if (q.get('timer')) focus.start('focus', 25);
    const game = q.get('game');
    if (game) setTimeout(() => activity.show('game', { name: game, exe: 'game.exe' }, 1e9), 300);
    if (q.get('missed'))
      activity.show('missed', { title: 'While you were playing', items: [{ project: 'meetbot', text: 'is done' }, { project: 'holm', text: 'needs you' }], count: 2 }, 1e9);
    const front = q.get('front');
    if (front) fit.set({ maximized: true, app: front });
    if (q.get('ended')) activity.show('gameover', { name: 'Valorant', exe: 'valorant.exe', durationMs: 4_320_000 }, 1e9);
  }
</script>

<svelte:window onpointermove={(e) => (pointer = { x: e.clientX, y: e.clientY })} onpointerdown={() => expanded && fit.used()} onwheel={() => expanded && fit.used()} onkeydown={() => expanded && fit.used()} />

<div
  class="stage"
  style="--d:{motion.duration}ms;--e:{motion.easing};--h:{compactH}px;transform:scale({s.scale});padding-top:{notch ? 0 : s.offsetY}px"
>
  <div
    class="island {style} {mode}"
    class:music={live === 'music' && media.now?.playing}
    bind:this={islandEl}
    style="width:{size.w}px;height:{size.h}px;--r:{radius}px"
    role="button"
    tabindex="-1"
    aria-label="Holm"
    onpointerenter={() => !inTauri && setHover(true)}
    onpointerleave={() => !inTauri && setHover(false)}
    onclick={() => (mode === 'compact' || mode === 'peek') && open('click')}
    onkeydown={(e) => e.key === 'Escape' && close()}
    ontransitionend={(e) => e.target === islandEl && reportRect()}
  >
    {#if notch}
      <span class="ear l" aria-hidden="true"></span>
      <span class="ear r" aria-hidden="true"></span>
    {/if}
    <span class="halo" aria-hidden="true"></span>
    <div class="body">
      {#key viewKey}
        <div class="view" use:measure={viewKey} in:enter out:leave>
          {#if mode === 'expanded'}
            <Expanded bind:page bind:pinned />
          {:else if mode === 'approval' && approval}
            <ApprovalCard {approval} more={agents.approvals.length - 1} />
          {:else if mode === 'peek' && activity.peek}
            <PeekView peek={activity.peek} />
          {:else if mode === 'drop'}
            <div class="drop-wrap"><div class="drop"><FolderDown size={22} /><span>Drop to keep on the shelf</span></div></div>
          {:else}
            <CompactView primary={live} line={lineNow} />
          {/if}
        </div>
      {/key}
    </div>

    {#if showBubble}
      <button
        class="bubble"
        bind:this={bubbleEl}
        aria-label="Open {secondary}"
        onclick={(e) => {
          e.stopPropagation();
          open('click', secondary === 'agent' ? 'agents' : secondary === 'timer' ? 'focus' : 'music');
        }}
        in:pop
        out:pop
      >
        {#if secondary === 'agent' && agents.focus}
          <AgentGlyph status={agents.focus.status} size={15} />
        {:else if secondary === 'timer'}
          <Ring value={focus.progress(clock.now) * 100} size={18} stroke={2.4} />
        {:else if secondary === 'music' && media.now}
          <Art src={media.now.art} size={22} radius={11} />
        {/if}
      </button>
    {/if}
  </div>
</div>

<script lang="ts" module>
  import { cubicOut } from 'svelte/easing';

  export function enter(_: Element) {
    return {
      delay: 60,
      duration: 260,
      easing: cubicOut,
      css: (t: number) => `opacity:${t};filter:blur(${(1 - t) * 6}px);transform:scale(${0.97 + 0.03 * t})`,
    };
  }

  export function leave(_: Element) {
    return {
      duration: 110,
      css: (t: number) => `opacity:${t};filter:blur(${(1 - t) * 4}px)`,
    };
  }

  export function pop(_: Element) {
    return {
      duration: 280,
      easing: cubicOut,
      css: (t: number) => `opacity:${t};transform:translateX(${(1 - t) * -14}px) scale(${0.6 + 0.4 * t})`,
    };
  }
</script>

<style>
  .stage {
    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    transform-origin: top center;
    pointer-events: none;
  }
  .island {
    position: relative;
    pointer-events: auto;
    transition:
      width var(--d) var(--e),
      height var(--d) var(--e);
    cursor: default;
    outline: none;
  }
  .body {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: var(--island);
    border-radius: var(--r);
    transition:
      border-radius var(--d) var(--e),
      box-shadow 0.4s;
    box-shadow:
      inset 0 0 0 1px rgb(from var(--rim-color) r g b / calc(alpha * var(--rim))),
      var(--shadow);
  }
  .notch .body {
    border-top-left-radius: 0;
    border-top-right-radius: 0;
    box-shadow:
      inset 0 -1px 0 0 rgb(from var(--rim-color) r g b / calc(alpha * var(--rim))),
      var(--shadow);
  }
  .notch.compact .body {
    box-shadow: inset 0 -1px 0 0 rgb(from var(--rim-color) r g b / calc(alpha * var(--rim) * 0.6));
  }
  .halo {
    position: absolute;
    inset: 0;
    border-radius: var(--r);
    box-shadow: 0 6px 26px -6px var(--accent);
    opacity: 0;
    transition:
      opacity 0.6s,
      border-radius var(--d) var(--e);
  }
  .notch .halo {
    border-top-left-radius: 0;
    border-top-right-radius: 0;
  }
  .island.music .halo {
    opacity: 0.3;
  }
  .island.expanded .halo {
    opacity: 0;
  }
  .ear {
    position: absolute;
    top: 0;
    width: 12px;
    height: 12px;
  }
  .ear.l {
    left: -12px;
    background: radial-gradient(circle at 0 100%, transparent 11.5px, var(--island) 12px);
  }
  .ear.r {
    right: -12px;
    background: radial-gradient(circle at 100% 100%, transparent 11.5px, var(--island) 12px);
  }
  .view {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    margin: 0 auto;
    width: max-content;
  }
  .line .body {
    box-shadow: none;
  }
  .bubble {
    position: absolute;
    top: 0;
    left: calc(100% + 8px);
    width: var(--h);
    height: var(--h);
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--island);
    box-shadow:
      inset 0 0 0 1px rgb(from var(--rim-color) r g b / calc(alpha * var(--rim))),
      var(--shadow);
  }
  .pill .bubble,
  .notch .bubble {
    top: 0;
  }
  .drop-wrap {
    padding: 10px;
  }
  .drop {
    width: 420px;
    height: 110px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    border-radius: 18px;
    border: 1.5px dashed var(--ink-3);
    color: var(--ink-2);
    font-weight: 600;
  }
</style>
