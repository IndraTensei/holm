<script lang="ts">
  // Every AI coding tool on this PC, found on its own, with what each used today.
  import { RefreshCw } from '@lucide/svelte';
  import Toggle from './Toggle.svelte';
  import Field from './Field.svelte';
  import Choice from './Choice.svelte';
  import Ring from '../ui/Ring.svelte';
  import { settings } from '../stores/settings.svelte';
  import { system } from '../stores/system.svelte';
  import { tools, shown, setShown } from '../stores/tools.svelte';
  import { clock } from '../stores/clock.svelte';
  import { send } from '../bridge';
  import { ago, tokens, until } from '../format';
  import type { Tool } from '../types';

  const s = $derived(settings.v);
  $effect(() => tools.use());

  /** A colour per tool for its monogram: hues only, no logos. */
  const HUE: Record<string, number> = {
    claude: 22, codex: 160, antigravity: 215, gemini: 250, copilot: 280, cline: 185,
    amp: 340, qwen: 265, crush: 320, droid: 30, kiro: 270, goose: 45, aider: 120,
  };
  const mono = (t: Tool) => t.name.replace(/^GitHub /, '').slice(0, 2);
  const tint = (t: Tool) => (t.id in HUE ? `hsl(${HUE[t.id]} 75% 62%)` : 'hsl(220 8% 72%)');

  const found = $derived(tools.installed);
  const missing = $derived(tools.list.filter((t) => !t.installed));
  const active = $derived(found.filter((t) => t.running).length);
  const totalTokens = $derived(found.reduce((n, t) => n + (t.today?.tokens ?? 0), 0));
  const totalSessions = $derived(found.reduce((n, t) => n + (t.today?.sessions ?? 0), 0));

  function status(t: Tool, now: number): string {
    if (t.running) return 'Active now';
    if (t.lastUsed) return `Used ${ago(t.lastUsed, now)}`;
    return 'Installed, not used yet';
  }

  function today(t: Tool): string {
    const d = t.today;
    if (!d || (!d.sessions && !d.tokens)) return 'Nothing today';
    const parts: string[] = [];
    if (d.tokens) parts.push(`${tokens(d.tokens)} tokens`);
    if (d.sessions) parts.push(`${d.sessions} ${d.sessions === 1 ? 'session' : 'sessions'}`);
    if (d.cost) parts.push(`$${d.cost.toFixed(2)}`);
    return parts.join(' · ');
  }

  const quota = (id: string) => system.quotas.find((q) => q.provider === id && q.ok);

  function toggle(id: string, on: boolean) {
    setShown(id, on);
    if (id === 'claude' || id === 'codex') setTimeout(() => system.refreshQuotas(), 200);
  }

  function rescan() {
    void tools.scan();
    send('quotas_refresh');
  }
</script>

<section>
  <h2 class="display">AI tools</h2>
  <p class="lead">Every coding agent on this PC, found on its own. Usage is read from the files each tool already keeps, and nothing leaves this PC.</p>

  <div class="stats">
    <div class="stat"><span class="v num">{found.length}</span><span class="k">found</span></div>
    <div class="stat"><span class="v num" class:live={active > 0}>{active}</span><span class="k">active now</span></div>
    <div class="stat"><span class="v num">{tokens(totalTokens)}</span><span class="k">tokens today</span></div>
    <div class="stat"><span class="v num">{totalSessions}</span><span class="k">sessions today</span></div>
  </div>

  {#if found.length}
    <div class="cards">
      {#each found as t (t.id)}
        {@const q = t.plan ? quota(t.id) : undefined}
        <article class="card" class:on={shown(t.id)} style="--tint:{tint(t)}">
          <header>
            <span class="mono display" aria-hidden="true">{mono(t)}</span>
            <div class="who">
              <div class="name">{t.name}</div>
              <div class="state" class:live={t.running}><i></i>{status(t, clock.now)}</div>
            </div>
            <Toggle label="Show {t.name} on the Agents page" checked={shown(t.id)} onchange={(v) => toggle(t.id, v)} />
          </header>
          <div class="today num">{today(t)}</div>
          {#if q}
            <div class="limits">
              {#each q.items.slice(0, 3) as item (item.label)}
                <div class="limit">
                  <Ring value={item.percent} size={30} stroke={3} />
                  <div>
                    <div class="l-label">{item.label}</div>
                    <div class="l-val num">{Math.round(item.percent)}%{#if item.resetsAt}<span>, resets in {until(item.resetsAt, clock.now)}</span>{/if}</div>
                  </div>
                </div>
              {/each}
            </div>
          {:else if t.plan && shown(t.id)}
            <div class="hint">Plan limits appear after the next check.</div>
          {/if}
        </article>
      {/each}
    </div>
  {:else if tools.scannedAt}
    <p class="empty">No AI coding tools found on this PC yet. Install one and it shows up here.</p>
  {:else}
    <p class="empty">Looking for AI coding tools…</p>
  {/if}

  {#if missing.length}
    <h3>Also recognised</h3>
    <p class="lead">Install any of these and it appears above on the next scan.</p>
    <div class="chips">
      {#each missing as t (t.id)}<span class="chip">{t.name}</span>{/each}
    </div>
  {/if}

  <h3>Checking</h3>
  <Field label="Plan limits every" hint="Read from the copy Claude Code and Codex keep on this PC, so checking often costs nothing. Their usage service is asked only when that copy is over 15 minutes old, at most every 5 minutes.">
    <Choice
      label="Plan limits every"
      value={s.quotaMinutes}
      options={[
        { value: 1, label: '1 min' },
        { value: 2, label: '2 min' },
        { value: 5, label: '5 min' },
        { value: 15, label: '15 min' },
      ]}
      onchange={(v) => settings.set({ quotaMinutes: v })}
    />
  </Field>
  <div class="foot">
    <span>{tools.scannedAt ? `Scanned ${ago(tools.scannedAt, clock.now)}` : 'Scanning…'}</span>
    <button class="ghost" disabled={tools.scanning} onclick={rescan}>
      <span class="icon" class:spin={tools.scanning}><RefreshCw size={13} /></span>Scan again
    </button>
  </div>
</section>

<style>
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    margin: 18px 0 16px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 14px 16px;
    border-radius: 14px;
    background: var(--fill-1);
  }
  .v {
    font-family: var(--font-display);
    font-size: var(--t-xl);
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .v.live {
    color: var(--ok);
  }
  .k {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
    gap: 10px;
  }
  .card {
    position: relative;
    padding: 14px 14px 14px 16px;
    border-radius: 16px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--line);
    overflow: hidden;
    transition: box-shadow 0.2s var(--ease-out);
  }
  .card::before {
    content: '';
    position: absolute;
    inset: 0 auto 0 0;
    width: 3px;
    background: var(--tint);
    opacity: 0.25;
    transition: opacity 0.2s var(--ease-out);
  }
  .card.on::before {
    opacity: 1;
  }
  .card.on {
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--tint) 35%, transparent);
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .mono {
    flex: none;
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 11px;
    background: color-mix(in srgb, var(--tint) 18%, transparent);
    color: var(--tint);
    font-weight: 700;
    font-size: var(--t-md);
    letter-spacing: -0.02em;
  }
  .who {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-weight: 650;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .state i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ink-4);
  }
  .state.live {
    color: var(--ok);
  }
  .state.live i {
    background: var(--ok);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--ok) 25%, transparent);
  }
  .today {
    margin: 12px 0 0;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .limits {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--line);
  }
  .limit {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .l-label {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .l-val {
    font-weight: 650;
    font-size: var(--t-sm);
  }
  .l-val span {
    font-weight: 400;
    color: var(--ink-3);
  }
  .hint,
  .empty {
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .hint {
    margin-top: 10px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    padding: 5px 10px;
    border-radius: 999px;
    background: var(--fill-1);
    color: var(--ink-3);
    font-size: var(--t-sm);
  }
  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 14px;
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .foot button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .icon {
    display: inline-grid;
  }
  .spin {
    animation: spin 0.9s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (max-width: 640px) {
    .stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
