<script lang="ts">
  import { Copy, Check, Trash2, Plug } from '@lucide/svelte';
  import AgentGlyph from '../ui/AgentGlyph.svelte';
  import Ring from '../ui/Ring.svelte';
  import ApprovalCard from '../views/ApprovalCard.svelte';
  import { agents } from '../stores/agents.svelte';
  import { system } from '../stores/system.svelte';
  import { tools, shown } from '../stores/tools.svelte';
  import { clock } from '../stores/clock.svelte';
  import { call, send } from '../bridge';
  import { ago, clockTime, until, tokens } from '../format';
  import type { AgentSession, HookStatus } from '../types';

  let selected = $state<string | null>(null);
  let copied = $state(false);
  let hooks = $state<HookStatus | null>(null);

  $effect(() => (agents.working.length ? clock.second() : undefined));
  // Other AI tools the owner chose to see (Settings → AI tools): what each did today.
  $effect(() => tools.use());
  const quotas = $derived(system.quotas.filter((q) => shown(q.provider)));
  const others = $derived(tools.installed.filter((t) => !t.plan && shown(t.id)));
  $effect(() => {
    void call<HookStatus>('hooks_status').then((h) => (hooks = h)).catch(() => {});
  });

  const current = $derived(agents.sessions.find((s) => s.id === selected) ?? agents.sessions[0] ?? null);

  function statusLine(s: AgentSession): string {
    if (s.status === 'waiting') return 'Waiting for you';
    if (s.status === 'working') return s.activity?.target ? `${s.activity.verb} ${s.activity.target}` : 'Thinking';
    if (s.status === 'done') return `Finished ${ago(s.updatedAt, clock.now)}`;
    if (s.status === 'error') return 'Stopped with an error';
    if (s.status === 'ended') return 'Session closed';
    return 'Idle';
  }

  async function copy(text: string) {
    send('clipboard_copy', { text });
    copied = true;
    setTimeout(() => (copied = false), 1400);
  }

  async function install() {
    hooks = await call<HookStatus>('hooks_install', { mode: 'http' }).catch(() => hooks);
  }
</script>

<div class="agents">
  {#each agents.approvals.slice(0, 1) as a (a.id)}
    <ApprovalCard approval={a} more={agents.approvals.length - 1} compact />
  {/each}

  {#if agents.sessions.length === 0}
    <div class="empty">
      <p class="display">No agent sessions yet</p>
      {#if hooks && !hooks.installed}
        <span>Connect Claude Code once and every session shows up here: what it reads, edits and runs, and when it needs you.</span>
        <button onclick={install}><Plug size={14} /> Connect Claude Code</button>
      {:else}
        <span>Start Claude Code in any terminal. Sessions appear here as soon as they do something.</span>
      {/if}
    </div>
  {:else}
    <div class="split">
      <ul class="list">
        {#each agents.sessions as s (s.id)}
          <li>
            <button class:on={current?.id === s.id} onclick={() => (selected = s.id)}>
              <AgentGlyph status={s.status} size={14} />
              <span class="name">
                <span class="project">{s.project}</span>
                <span class="state">{statusLine(s)}</span>
              </span>
              {#if s.status === 'working' || s.status === 'waiting'}
                <span class="num t">{clockTime((clock.now - s.turnStartedAt) / 1000)}</span>
              {/if}
            </button>
          </li>
        {/each}
        {#if agents.sessions.some((s) => s.status === 'done' || s.status === 'ended' || s.status === 'idle')}
          <li><button class="clear" onclick={() => agents.clear()}><Trash2 size={13} /> Clear finished</button></li>
        {/if}
      </ul>

      {#if current}
        <div class="detail">
          {#if current.prompt}<p class="request">{current.prompt}</p>{/if}
          {#if current.status === 'done' && current.message}
            <div class="answer">
              <p>{current.message}</p>
              <button class="copy" onclick={() => copy(current.message)} aria-label="Copy answer">
                {#if copied}<Check size={13} /> Copied{:else}<Copy size={13} /> Copy{/if}
              </button>
            </div>
          {/if}
          <ol class="log">
            {#each current.log.filter((e) => e.kind !== 'prompt').slice(0, 6) as entry}
              <li class={entry.kind}>
                <span class="text">{entry.text}</span>
                <span class="num when">{ago(entry.at, clock.now)}</span>
              </li>
            {/each}
          </ol>
          <div class="facts">
            {current.tools} tool {current.tools === 1 ? 'call' : 'calls'}{#if current.edited.length}, edited {current.edited.slice(0, 3).join(', ')}{current.edited.length > 3 ? ` and ${current.edited.length - 3} more` : ''}{/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if quotas.length || others.length}
    <div class="quotas">
      {#each quotas as p (p.provider)}
        {#if p.ok}
          {#each p.items.slice(0, 3) as q}
            <div class="q" class:stale={p.error} title={p.error ? `${p.error}. Last updated ${ago(p.fetchedAt, clock.now)}.` : undefined}>
              <Ring value={q.percent} size={26} stroke={3} />
              <div>
                <div class="q-label">{p.provider === 'claude' ? 'Claude' : 'Codex'} {q.label.toLowerCase()}</div>
                <div class="q-meta num">{Math.round(q.percent)}% used{#if p.error}, updated {ago(p.fetchedAt, clock.now)}{:else if q.resetsAt}, resets in {until(q.resetsAt, clock.now)}{/if}</div>
              </div>
            </div>
          {/each}
        {:else}
          <div class="q off">
            <Ring value={0} size={26} stroke={3} />
            <div>
              <div class="q-label">{p.provider === 'claude' ? 'Claude' : 'Codex'}</div>
              <div class="q-meta">{p.error}</div>
            </div>
          </div>
        {/if}
      {/each}
      {#each others as t (t.id)}
        <div class="q tool" class:stale={!t.running}>
          <span class="dot" class:live={t.running}></span>
          <div>
            <div class="q-label">{t.name}</div>
            <div class="q-meta num">
              {#if t.today?.tokens}{tokens(t.today.tokens)} tokens today{:else if t.today?.sessions}{t.today.sessions} {t.today.sessions === 1 ? 'session' : 'sessions'} today{:else if t.lastUsed}used {ago(t.lastUsed, clock.now)}{:else}not used yet{/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .agents {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 2px;
  }
  .split {
    display: grid;
    grid-template-columns: 210px 1fr;
    gap: 14px;
    min-height: 150px;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 230px;
    overflow-y: auto;
  }
  .list button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 12px;
    text-align: left;
    transition: background 0.15s;
  }
  .list button:hover {
    background: var(--fill-1);
  }
  .list button.on {
    background: var(--fill-2);
  }
  .name {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .project {
    font-weight: 600;
    font-size: var(--t-md);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .state {
    font-size: var(--t-xs);
    color: var(--ink-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .t {
    font-size: var(--t-xs);
    color: var(--ink-2);
  }
  .clear {
    color: var(--ink-3);
    font-size: var(--t-xs);
    gap: 6px !important;
  }
  .detail {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .request {
    margin: 0;
    padding-left: 10px;
    border-left: 2px solid var(--accent);
    font-size: var(--t-sm);
    line-height: 1.45;
    color: var(--ink-2);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .answer {
    position: relative;
    padding: 10px 12px 30px;
    border-radius: 12px;
    background: var(--fill-1);
  }
  .answer p {
    margin: 0;
    font-size: var(--t-sm);
    line-height: 1.5;
    max-height: 92px;
    overflow-y: auto;
    user-select: text;
    white-space: pre-wrap;
  }
  .copy {
    position: absolute;
    right: 8px;
    bottom: 6px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: var(--t-xs);
    color: var(--ink-2);
    padding: 3px 7px;
    border-radius: 999px;
  }
  .copy:hover {
    background: var(--fill-2);
  }
  .log {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .log li {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .log li::before {
    content: '';
    flex: none;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--ink-4);
    transform: translateY(-2px);
  }
  .log li:first-child {
    color: var(--ink);
  }
  .log li:first-child::before {
    background: var(--accent);
  }
  .log li.error::before {
    background: var(--err);
  }
  .log li.ask::before {
    background: var(--warn);
  }
  .log li.done::before {
    background: var(--ok);
  }
  .log .text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .when {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .facts {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .quotas {
    display: flex;
    flex-wrap: wrap;
    gap: 18px;
    padding: 12px 6px 2px;
    border-top: 1px solid var(--line);
  }
  .q {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .q-label {
    font-size: var(--t-sm);
    font-weight: 600;
  }
  .q-meta {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .tool .dot {
    width: 26px;
    height: 26px;
    flex: none;
    border-radius: 50%;
    box-shadow: inset 0 0 0 3px var(--fill-2);
    display: grid;
    place-items: center;
  }
  .tool .dot.live {
    box-shadow: inset 0 0 0 3px color-mix(in srgb, var(--ok) 60%, transparent);
  }
  .q.stale {
    opacity: 0.7;
  }
  .empty {
    padding: 18px 8px 10px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }
  .empty p {
    margin: 0;
    font-size: var(--t-xl);
    font-weight: 650;
    letter-spacing: -0.02em;
  }
  .empty span {
    color: var(--ink-2);
    font-size: var(--t-sm);
    line-height: 1.5;
    max-width: 54ch;
  }
  .empty button {
    margin-top: 8px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border-radius: 999px;
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 600;
    font-size: var(--t-sm);
  }
</style>
