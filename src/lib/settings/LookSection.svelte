<script lang="ts">
  import { Plus, Trash2 } from '@lucide/svelte';
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import Choice from './Choice.svelte';
  import { settings } from '../stores/settings.svelte';
  import { THEMES, FONTS, findTheme, fontStack, alpha, type Theme } from '../themes';

  const s = $derived(settings.v);
  const all = $derived([...THEMES, ...s.customThemes]);
  const active = $derived(findTheme(s.theme, s.customThemes));
  let editing = $state<Theme | null>(null);

  function duplicate() {
    const base = active;
    const theme: Theme = { ...base, id: `custom-${Date.now().toString(36)}`, name: `${base.name} (mine)`, custom: true, adaptive: false };
    void settings.set({ customThemes: [...s.customThemes, theme], theme: theme.id });
    editing = theme;
  }

  function saveEdit(patch: Partial<Theme>) {
    if (!editing) return;
    editing = { ...editing, ...patch };
    const list = s.customThemes.map((t) => (t.id === editing!.id ? editing! : t));
    void settings.set({ customThemes: list });
  }

  function remove(id: string) {
    void settings.set({ customThemes: s.customThemes.filter((t) => t.id !== id), theme: s.theme === id ? 'obsidian' : s.theme });
    if (editing?.id === id) editing = null;
  }

  const uiFonts = FONTS.filter((f) => !['instrument', 'fraunces'].includes(f.id));
</script>

<section>
  <h2 class="display">Look</h2>

  <div class="themes">
    {#each all as t (t.id)}
      <button
        class="theme"
        class:on={t.id === s.theme}
        aria-pressed={t.id === s.theme}
        onclick={() => {
          void settings.set({ theme: t.id });
          editing = t.custom ? t : null;
        }}
        style="--t-island:{t.island};--t-ink:{t.ink};--t-accent:{t.accent};--t-dim:{alpha(t.ink, 0.45)};--t-font:{fontStack(t.fontDisplay)}"
      >
        <span class="swatch">
          <span class="mini">
            <span class="dot"></span>
            <span class="bars"><i></i><i></i><i></i></span>
          </span>
          <span class="sample">Aa</span>
        </span>
        <span class="name">{t.name}</span>
      </button>
    {/each}
    <button class="theme add" onclick={duplicate}>
      <span class="swatch"><Plus size={20} /></span>
      <span class="name">Make my own</span>
    </button>
  </div>

  {#if editing}
    <div class="editor">
      <div class="ed-head">
        <input class="ed-name" value={editing.name} oninput={(e) => saveEdit({ name: (e.target as HTMLInputElement).value })} aria-label="Theme name" />
        <button class="danger" onclick={() => remove(editing!.id)}><Trash2 size={14} /> Delete</button>
      </div>
      <div class="colors">
        {#each [['island', 'Island'], ['ink', 'Text'], ['accent', 'Accent'], ['ok', 'Success'], ['warn', 'Attention'], ['err', 'Error']] as [key, label]}
          <label class="color">
            <input type="color" value={(editing as unknown as Record<string, string>)[key]} oninput={(e) => saveEdit({ [key]: (e.target as HTMLInputElement).value } as Partial<Theme>)} />
            <span>{label}</span>
          </label>
        {/each}
      </div>
      <Field label="Light theme" hint="Use dark text shades and shadows tuned for light islands.">
        <Toggle label="Light theme" checked={editing.mode === 'light'} onchange={(v) => saveEdit({ mode: v ? 'light' : 'dark' })} />
      </Field>
      <Field label="Edge light">
        <input type="range" min="0" max="1" step="0.05" value={editing.rim} oninput={(e) => saveEdit({ rim: Number((e.target as HTMLInputElement).value) })} />
      </Field>
    </div>
  {/if}

  <h3>Type</h3>
  <Field label="Text" hint="Used for everything you read on the island.">
    <select value={s.fontUi} onchange={(e) => settings.set({ fontUi: (e.target as HTMLSelectElement).value })} style="font-family:{fontStack(s.fontUi || active.fontUi)}">
      <option value="">Theme default ({FONTS.find((f) => f.id === active.fontUi)?.label})</option>
      {#each uiFonts as f (f.id)}<option value={f.id} style="font-family:{f.stack}">{f.label}</option>{/each}
    </select>
  </Field>
  <Field label="Display" hint="Clock, titles and big numbers.">
    <select value={s.fontDisplay} onchange={(e) => settings.set({ fontDisplay: (e.target as HTMLSelectElement).value })} style="font-family:{fontStack(s.fontDisplay || active.fontDisplay)}">
      <option value="">Theme default ({FONTS.find((f) => f.id === active.fontDisplay)?.label})</option>
      {#each FONTS as f (f.id)}<option value={f.id} style="font-family:{f.stack}">{f.label}</option>{/each}
    </select>
  </Field>
  <div class="specimen" style="--f-ui:{fontStack(s.fontUi || active.fontUi)};--f-display:{fontStack(s.fontDisplay || active.fontDisplay)}">
    <div class="sp-display num">9:41</div>
    <div class="sp-text">
      <strong>Lanterns Over Kochi</strong>
      <span>The Monsoon Office, 2:14 left. Editing commands.ts in qinshin-bot.</span>
    </div>
  </div>

  <h3>Island</h3>
  <Field label="Shape" hint="Notch sits on the top edge like a camera cut-out; pill floats below it; line stays out of the way until you hover.">
    <Choice
      label="Shape"
      value={s.style}
      options={[
        { value: 'notch', label: 'Notch' },
        { value: 'pill', label: 'Pill' },
        { value: 'line', label: 'Line' },
      ]}
      onchange={(v) => settings.set({ style: v })}
    />
  </Field>
  {#if s.style !== 'notch'}
    <Field label="Distance from the top">
      <input type="range" min="0" max="40" step="1" value={s.offsetY} oninput={(e) => settings.set({ offsetY: Number((e.target as HTMLInputElement).value) })} />
      <span class="num val">{s.offsetY} px</span>
    </Field>
  {/if}
  <Field label="Size">
    <input type="range" min="0.85" max="1.25" step="0.05" value={s.scale} oninput={(e) => settings.set({ scale: Number((e.target as HTMLInputElement).value) })} />
    <span class="num val">{Math.round(s.scale * 100)}%</span>
  </Field>
  <Field label="Colour from album art" hint="With Obsidian, the accent and the glow follow the cover of what is playing.">
    <Toggle label="Colour from album art" checked={s.adaptiveAccent} onchange={(v) => settings.set({ adaptiveAccent: v })} />
  </Field>
  <Field label="Motion">
    <Choice
      label="Motion"
      value={s.motion}
      options={[
        { value: 'spring', label: 'Springy' },
        { value: 'smooth', label: 'Smooth' },
        { value: 'reduced', label: 'Minimal' },
      ]}
      onchange={(v) => settings.set({ motion: v })}
    />
  </Field>

  <h3>Resting island</h3>
  <Field label="Clock"><Toggle label="Clock" checked={s.showClock} onchange={(v) => settings.set({ showClock: v })} /></Field>
  <Field label="24-hour time"><Toggle label="24-hour time" checked={s.clock24h} onchange={(v) => settings.set({ clock24h: v })} /></Field>
  <Field label="Plan usage ring" hint="The highest of your Claude or Codex limits, coloured as it fills.">
    <Toggle label="Plan usage ring" checked={s.showQuotaRing} onchange={(v) => settings.set({ showQuotaRing: v })} />
  </Field>
  <Field label="While music plays, show">
    <Choice
      label="While music plays"
      value={s.compactTitle}
      options={[
        { value: 'none', label: 'Cover only' },
        { value: 'title', label: 'Title' },
        { value: 'lyrics', label: 'Lyrics' },
      ]}
      onchange={(v) => settings.set({ compactTitle: v })}
    />
  </Field>
</section>

<style>
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(118px, 1fr));
    gap: 10px;
    margin-bottom: 8px;
  }
  .theme {
    display: flex;
    flex-direction: column;
    gap: 7px;
    padding: 6px;
    border-radius: 14px;
    text-align: left;
    transition: background 0.15s;
  }
  .theme:hover {
    background: var(--fill-1);
  }
  .swatch {
    position: relative;
    height: 74px;
    border-radius: 10px;
    background: var(--t-island, var(--fill-1));
    box-shadow: inset 0 0 0 1px var(--line);
    display: grid;
    place-items: center;
    overflow: hidden;
    color: var(--ink-3);
  }
  .on .swatch {
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .mini {
    position: absolute;
    top: 10px;
    left: 50%;
    transform: translateX(-50%);
    width: 70px;
    height: 18px;
    border-radius: 9px;
    background: color-mix(in srgb, var(--t-ink) 10%, var(--t-island));
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 6px;
  }
  .mini .dot {
    width: 8px;
    height: 8px;
    border-radius: 3px;
    background: var(--t-accent);
  }
  .bars {
    display: flex;
    gap: 2px;
    align-items: center;
  }
  .bars i {
    width: 2px;
    height: 8px;
    border-radius: 1px;
    background: var(--t-accent);
  }
  .bars i:nth-child(2) {
    height: 5px;
  }
  .sample {
    position: absolute;
    bottom: 6px;
    left: 9px;
    font-family: var(--t-font);
    font-size: 22px;
    font-weight: 600;
    color: var(--t-ink);
    letter-spacing: -0.02em;
  }
  .name {
    font-size: var(--t-sm);
    font-weight: 550;
    padding: 0 2px;
  }
  .add .swatch {
    background: transparent;
    box-shadow: inset 0 0 0 1.5px var(--ink-4);
  }
  .editor {
    margin: 8px 0 4px;
    padding: 14px 16px 4px;
    border-radius: 14px;
    background: var(--fill-1);
  }
  .ed-head {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .ed-name {
    flex: 1;
    height: 32px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: var(--fill-1);
    font-weight: 600;
  }
  .danger {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--err);
    font-size: var(--t-sm);
  }
  .colors {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    padding: 14px 0 6px;
  }
  .color {
    display: flex;
    align-items: center;
    gap: 9px;
    font-size: var(--t-sm);
  }
  .color input {
    width: 32px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    background: none;
    cursor: pointer;
  }
  .specimen {
    margin: 10px 0 4px;
    padding: 16px 18px;
    border-radius: 14px;
    background: var(--island);
    box-shadow: inset 0 0 0 1px var(--line);
    display: flex;
    align-items: center;
    gap: 18px;
  }
  .sp-display {
    font-family: var(--f-display);
    font-size: 40px;
    font-weight: 700;
    letter-spacing: -0.04em;
  }
  .sp-text {
    font-family: var(--f-ui);
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .sp-text strong {
    font-size: var(--t-lg);
    color: var(--ink);
  }
  .val {
    width: 52px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
</style>
