<script lang="ts">
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import Choice from './Choice.svelte';
  import { settings } from '../stores/settings.svelte';
  import { system } from '../stores/system.svelte';
  import { call, send } from '../bridge';
  import type { HookStatus } from '../types';

  const s = $derived(settings.v);
  let hooks = $state<HookStatus | null>(null);
  let mode = $state<'http' | 'curl'>('http');
  let error = $state('');
  let busy = $state(false);

  async function refresh() {
    hooks = await call<HookStatus>('hooks_status').catch(() => null);
    if (hooks?.mode === 'command') mode = 'curl';
  }
  $effect(() => {
    void refresh();
  });

  async function install() {
    busy = true;
    error = '';
    try {
      hooks = await call<HookStatus>('hooks_install', { mode });
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function uninstall() {
    busy = true;
    error = '';
    try {
      hooks = await call<HookStatus>('hooks_uninstall');
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }
</script>

<section>
  <h2 class="display">Agents</h2>
  <p class="lead">
    See what Claude Code is doing in every terminal, answer its permission requests from the island, and hear when it finishes. Everything stays on this PC.
  </p>

  <div class="connect">
    <div class="c-head">
      <span class="status" class:ok={hooks?.installed} class:warn={hooks && !hooks.installed && hooks.events > 0}></span>
      <div>
        <div class="c-title">Claude Code</div>
        <div class="c-text">
          {#if !hooks}Checking…
          {:else if hooks.installed}Connected through {hooks.mode === 'http' ? 'HTTP hooks' : 'a curl hook'} on port {hooks.port}.
          {:else if hooks.events > 0}Partly connected ({hooks.events} of 12 events). Connect again to repair.
          {:else if !hooks.claudeFound}Claude Code's settings folder was not found. Run claude once, then connect.
          {:else}Not connected yet.{/if}
        </div>
      </div>
      {#if hooks?.installed}
        <button class="ghost" disabled={busy} onclick={uninstall}>Disconnect</button>
      {:else}
        <button class="primary" disabled={busy} onclick={install}>Connect Claude Code</button>
      {/if}
    </div>
    <p class="c-note">
      Adds hooks to <code>{hooks?.path ?? '~/.claude/settings.json'}</code> and keeps a backup next to it. Your other hooks and settings are left as they are. Sessions that are already open pick up the change after a restart.
    </p>
    {#if error}<p class="c-error">{error}</p>{/if}
  </div>

  <Field label="Hook type" hint="HTTP hooks need a recent Claude Code. Use curl for older versions.">
    <Choice label="Hook type" value={mode} options={[{ value: 'http', label: 'HTTP' }, { value: 'curl', label: 'curl' }]} onchange={(v) => (mode = v)} />
  </Field>
  <Field label="Open permission requests" hint="The island shows Allow and Deny the moment Claude Code asks.">
    <Toggle label="Open permission requests" checked={s.autoOpenApprovals} onchange={(v) => settings.set({ autoOpenApprovals: v })} />
  </Field>
  <Field label="Time to answer" hint="After this, the question goes back to the terminal as usual.">
    <input type="range" min="15" max="300" step="15" value={s.approvalTimeout} oninput={(e) => settings.set({ approvalTimeout: Number((e.target as HTMLInputElement).value) })} />
    <span class="num val">{s.approvalTimeout} s</span>
  </Field>
  <Field label="Sounds" hint="A soft chime when an agent finishes or asks.">
    <Toggle label="Sounds" checked={s.agentSounds} onchange={(v) => settings.set({ agentSounds: v })} />
  </Field>

  <h3>Plan usage</h3>
  <p class="lead">Claude Code and Codex limits, and every other AI coding agent on this PC, are in Harnesses.</p>
</section>

<style>
  .connect {
    margin: 6px 0 12px;
    padding: 14px 16px;
    border-radius: 14px;
    background: var(--fill-1);
  }
  .c-head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .c-head > div {
    flex: 1;
  }
  .status {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--ink-4);
  }
  .status.ok {
    background: var(--ok);
  }
  .status.warn {
    background: var(--warn);
  }
  .c-title {
    font-weight: 600;
  }
  .c-text {
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .c-note {
    margin: 10px 0 0;
    font-size: var(--t-sm);
    line-height: 1.55;
    color: var(--ink-3);
  }
  code {
    font-family: 'JetBrains Mono Variable', Consolas, monospace;
    font-size: 12px;
    color: var(--ink-2);
  }
  .c-error {
    margin: 8px 0 0;
    font-size: var(--t-sm);
    color: var(--err);
  }
  .val {
    width: 44px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
</style>
