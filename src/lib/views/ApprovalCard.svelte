<script lang="ts">
  // A permission request from Claude Code, answered without leaving what you are doing.
  import { TerminalSquare, FilePen, Globe, Wrench } from '@lucide/svelte';
  import Ring from '../ui/Ring.svelte';
  import type { Approval } from '../types';
  import { agents } from '../stores/agents.svelte';
  import { clock } from '../stores/clock.svelte';

  let { approval, more = 0, compact = false }: { approval: Approval; more?: number; compact?: boolean } = $props();

  $effect(() => clock.second());

  const left = $derived(Math.max(0, approval.expiresAt - clock.now));
  const total = $derived(Math.max(1, approval.expiresAt - approval.createdAt));
  const kind = $derived(
    /^(Bash|PowerShell)$/.test(approval.tool)
      ? 'run'
      : /^(Edit|MultiEdit|Write|NotebookEdit)$/.test(approval.tool)
        ? 'edit'
        : /^Web/.test(approval.tool)
          ? 'web'
          : 'tool',
  );
  const ask = $derived(
    kind === 'run' ? 'wants to run' : kind === 'edit' ? 'wants to change a file' : kind === 'web' ? 'wants to go online' : `wants to use ${approval.tool}`,
  );
</script>

<div class="card" class:compact>
  <div class="head">
    <span class="icon">
      {#if kind === 'run'}<TerminalSquare size={16} />{:else if kind === 'edit'}<FilePen size={16} />{:else if kind === 'web'}<Globe size={16} />{:else}<Wrench size={16} />{/if}
    </span>
    <div class="who">
      <span class="project display">{approval.project}</span>
      <span class="ask">{ask}</span>
    </div>
    <Ring value={(left / total) * 100} size={18} stroke={2.2} color="var(--warn)" />
  </div>
  <pre class="detail">{approval.detail || approval.title}</pre>
  <div class="actions">
    {#if more > 0}<span class="more">{more} more waiting</span>{/if}
    <button class="deny" onclick={(e) => { e.stopPropagation(); agents.decide(approval.id, false); }}>Deny</button>
    <button class="allow" onclick={(e) => { e.stopPropagation(); agents.decide(approval.id, true); }}>Allow</button>
  </div>
</div>

<style>
  .card {
    width: 440px;
    padding: 14px 14px 12px;
  }
  .card.compact {
    width: 100%;
    padding: 12px;
    border-radius: 18px;
    background: var(--fill-1);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    flex: none;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--warn) 18%, transparent);
    color: var(--warn);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
  }
  .project {
    font-size: var(--t-lg);
    font-weight: 650;
    letter-spacing: -0.015em;
  }
  .ask {
    font-size: var(--t-sm);
    color: var(--ink-2);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detail {
    margin: 10px 0 12px;
    padding: 9px 11px;
    max-height: 84px;
    overflow: auto;
    border-radius: 10px;
    background: var(--fill-1);
    font-family: 'JetBrains Mono Variable', Consolas, monospace;
    font-size: 12px;
    line-height: 1.5;
    color: var(--ink);
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
  }
  .compact .detail {
    background: var(--fill-1);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }
  .more {
    margin-right: auto;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  button {
    height: 32px;
    padding: 0 18px;
    border-radius: 999px;
    font-size: var(--t-sm);
    font-weight: 600;
    transition: transform 0.12s, filter 0.15s, background 0.15s;
  }
  button:active {
    transform: scale(0.95);
  }
  .deny {
    background: var(--fill-2);
    color: var(--ink);
  }
  .deny:hover {
    background: var(--fill-3);
  }
  .allow {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .allow:hover {
    filter: brightness(1.08);
  }
</style>
