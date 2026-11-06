<script lang="ts">
  import { ArrowUp, ArrowDown } from '@lucide/svelte';
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import Choice from './Choice.svelte';
  import { settings, DEFAULTS, type PageId } from '../stores/settings.svelte';
  import { call } from '../bridge';
  import type { MonitorInfo } from '../types';

  const s = $derived(settings.v);
  let monitors = $state<MonitorInfo[]>([]);
  let recording = $state(false);
  let autostart = $state(false);

  $effect(() => {
    void call<MonitorInfo[]>('monitors_list').then((m) => (monitors = m ?? [])).catch(() => {});
    void call<boolean>('autostart_get').then((v) => (autostart = Boolean(v))).catch(() => {});
  });

  const ALL: { id: PageId; label: string }[] = [
    { id: 'home', label: 'Home' },
    { id: 'music', label: 'Music' },
    { id: 'agents', label: 'Agents' },
    { id: 'system', label: 'System' },
    { id: 'shelf', label: 'Shelf' },
    { id: 'focus', label: 'Focus' },
  ];
  const ordered = $derived([...s.pages.map((id) => ALL.find((p) => p.id === id)!).filter(Boolean), ...ALL.filter((p) => !s.pages.includes(p.id))]);

  function move(id: PageId, dir: -1 | 1) {
    const list = [...s.pages];
    const i = list.indexOf(id);
    const j = i + dir;
    if (i < 0 || j < 0 || j >= list.length) return;
    [list[i], list[j]] = [list[j], list[i]];
    void settings.set({ pages: list });
  }

  function togglePage(id: PageId, on: boolean) {
    const list = on ? [...s.pages, id] : s.pages.filter((p) => p !== id);
    if (list.length) void settings.set({ pages: list });
  }

  function keyName(e: KeyboardEvent): string | null {
    const c = e.code;
    if (/^Key[A-Z]$/.test(c)) return c.slice(3);
    if (/^Digit\d$/.test(c)) return c.slice(5);
    if (/^F\d{1,2}$/.test(c)) return c;
    const map: Record<string, string> = { Space: 'Space', Backquote: '`', Slash: '/', Period: '.', Comma: ',', Semicolon: ';', Enter: 'Enter', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right' };
    return map[c] ?? null;
  }

  function record(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    if (e.key === 'Escape') {
      recording = false;
      return;
    }
    const key = keyName(e);
    if (!key) return;
    const mods = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', e.metaKey && 'Super'].filter(Boolean);
    if (!mods.length) return;
    recording = false;
    void settings.set({ hotkey: [...mods, key].join('+') });
  }
</script>

<section>
  <h2 class="display">General</h2>

  <Field label="Start with Windows" hint="The island is there as soon as you sign in.">
    <Toggle
      label="Start with Windows"
      checked={autostart}
      onchange={(v) => {
        autostart = v;
        void settings.set({ autostart: v });
      }}
    />
  </Field>

  <Field label="Graphics" hint="Software drawing uses about 60 MB less memory; hardware opens and closes the island a little more smoothly. Takes effect the next time Tensei Island starts.">
    <Choice
      label="Graphics"
      value={s.graphics}
      options={[
        { value: 'gpu', label: 'Hardware' },
        { value: 'software', label: 'Software' },
      ]}
      onchange={(v) => void settings.set({ graphics: v })}
    />
  </Field>

  <Field label="Shortcut" hint="Opens and closes the island from anywhere. Click, then press the keys.">
    <button class="key" class:rec={recording} onclick={() => (recording = true)} onkeydown={record} onblur={() => (recording = false)}>
      {recording ? 'Press keys…' : s.hotkey || 'None'}
    </button>
    {#if s.hotkey !== DEFAULTS.hotkey}<button class="link" onclick={() => settings.set({ hotkey: DEFAULTS.hotkey })}>Reset</button>{/if}
  </Field>

  <Field label="Open" hint="Hovering opens after a short pause, so passing the pointer by does nothing.">
    <Choice label="Open" value={s.openOn} options={[{ value: 'hover', label: 'On hover' }, { value: 'click', label: 'On click' }]} onchange={(v) => settings.set({ openOn: v })} />
  </Field>

  <Field label="Hover delay" hint="How long the pointer rests before the island opens.">
    <input type="range" min="0" max="600" step="20" value={s.hoverDelay} oninput={(e) => settings.set({ hoverDelay: Number((e.target as HTMLInputElement).value) })} />
    <span class="num val">{s.hoverDelay} ms</span>
  </Field>

  <Field label="Hide during full screen" hint="Games, videos and presentations are never covered.">
    <Toggle label="Hide during full screen" checked={s.hideInFullscreen} onchange={(v) => settings.set({ hideInFullscreen: v })} />
  </Field>

  <Field label="Thin line when a window is maximized" hint="Keeps the top of maximized apps clear; hover still opens it.">
    <Toggle label="Thin line when maximized" checked={s.lineWhenMaximized} onchange={(v) => settings.set({ lineWhenMaximized: v })} />
  </Field>

  {#if monitors.length > 1}
    <Field label="Display">
      <select value={s.monitor} onchange={(e) => settings.set({ monitor: Number((e.target as HTMLSelectElement).value) })}>
        {#each monitors as m (m.index)}
          <option value={m.index}>{m.primary ? 'Main display' : `Display ${m.index + 1}`} ({m.width}×{m.height})</option>
        {/each}
      </select>
    </Field>
  {/if}

  <h3>Pages</h3>
  <p class="lead">Which pages the open island shows, in this order.</p>
  <div class="pages">
    {#each ordered as p, i (p.id)}
      {@const on = s.pages.includes(p.id)}
      <div class="page" class:off={!on}>
        <Toggle label={p.label} checked={on} onchange={(v) => togglePage(p.id, v)} />
        <span>{p.label}</span>
        {#if on}
          <button aria-label="Move {p.label} up" disabled={i === 0} onclick={() => move(p.id, -1)}><ArrowUp size={14} /></button>
          <button aria-label="Move {p.label} down" disabled={i === s.pages.length - 1} onclick={() => move(p.id, 1)}><ArrowDown size={14} /></button>
        {/if}
      </div>
    {/each}
  </div>
</section>

<style>
  .key {
    min-width: 150px;
    height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--line);
    font-weight: 600;
    font-size: var(--t-sm);
  }
  .key.rec {
    box-shadow: inset 0 0 0 1.5px var(--accent);
    color: var(--accent);
  }
  .link {
    color: var(--accent);
    font-size: var(--t-sm);
  }
  .val {
    width: 56px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .pages {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .page {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 0;
  }
  .page span {
    flex: 1;
    font-weight: 550;
  }
  .page.off span {
    color: var(--ink-3);
  }
  .page button {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--ink-2);
  }
  .page button:hover {
    background: var(--fill-1);
  }
  .page button:disabled {
    opacity: 0.3;
  }
</style>
