<script lang="ts">
  // Settings → About: check for and install updates, and uninstall. See src-tauri/src/update.rs.
  import { onMount } from 'svelte';
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import { call, on, send } from '../bridge';
  import { settings } from '../stores/settings.svelte';
  import type { UninstallInfo, UpdateInfo, UpdateProgress } from '../types';

  const RELEASES = 'https://github.com/IndraTensei/holm/releases/latest';

  let phase = $state<'idle' | 'checking' | 'done' | 'installing' | 'error'>('idle');
  let info = $state<UpdateInfo | null>(null);
  let error = $state('');
  let progress = $state(0);
  let uninstallInfo = $state<UninstallInfo | null>(null);
  let confirming = $state(false);

  onMount(() => {
    void call<UninstallInfo>('uninstall_info').then((u) => (uninstallInfo = u)).catch(() => {});
    // With automatic checks on, opening About checks at once, so "Update" is ready to click.
    if (settings.v.autoUpdateCheck) void check();
    let stop: (() => void) | undefined;
    void on<UpdateProgress>('update://progress', (p) => (progress = p.total ? p.received / p.total : 0)).then((s) => (stop = s));
    return () => stop?.();
  });

  async function check() {
    phase = 'checking';
    error = '';
    try {
      info = await call<UpdateInfo>('update_check');
      phase = 'done';
    } catch (e) {
      error = String(e);
      phase = 'error';
    }
  }

  async function install() {
    phase = 'installing';
    progress = 0;
    error = '';
    try {
      // Holm quits once the installer starts; the installer opens it again when it's done.
      await call('update_install');
    } catch (e) {
      error = String(e);
      phase = 'error';
    }
  }

  const status = $derived.by(() => {
    if (phase === 'checking') return 'Checking…';
    if (phase === 'installing') return progress < 1 ? `Downloading the update… ${Math.round(progress * 100)}%` : 'Starting the installer. Holm will close and open again.';
    if (phase === 'error') return error;
    if (phase === 'done' && info) return info.available ? `Holm ${info.latest} is out.` : `You have the latest version.`;
    return '';
  });
</script>

<h3>Updates</h3>
<Field label="Holm {__APP_VERSION__}" hint={status || 'Check GitHub for a newer version.'}>
  {#if phase === 'done' && info?.available}
    <div class="row">
      <button class="ghost" onclick={() => send('open_url', { url: info!.url })}>What's new</button>
      {#if info.installed}
        <button class="primary" onclick={install}>Update to {info.latest}</button>
      {:else}
        <button class="primary" onclick={() => send('open_url', { url: RELEASES })}>Download</button>
      {/if}
    </div>
  {:else}
    <button class="ghost" disabled={phase === 'checking' || phase === 'installing'} onclick={check}>Check for updates</button>
  {/if}
</Field>
<Field label="Check automatically" hint="Once a day, Holm asks GitHub for its latest release and tells you when there's a new one. Nothing about you is sent.">
  <Toggle label="Check automatically" checked={settings.v.autoUpdateCheck} onchange={(v) => settings.set({ autoUpdateCheck: v })} />
</Field>

<h3>Uninstall</h3>
{#if uninstallInfo?.installed}
  <Field label="Remove Holm from this PC" hint="Opens Holm's uninstaller, which asks you to confirm and whether to delete your settings too. Holm closes first.">
    {#if confirming}
      <div class="row">
        <button class="ghost" onclick={() => (confirming = false)}>Cancel</button>
        <button class="primary danger" onclick={() => void call('uninstall').catch((e) => ((error = String(e)), (phase = 'error')))}>Uninstall</button>
      </div>
    {:else}
      <button class="ghost" onclick={() => (confirming = true)}>Uninstall Holm…</button>
    {/if}
  </Field>
{:else if uninstallInfo}
  <Field label="Remove the portable copy" hint="Turn off Start with Windows in General, quit Holm from the tray, then delete the exe. Your settings are in the folder this opens.">
    <button class="ghost" onclick={() => send('open_path', { path: uninstallInfo!.dataDir })}>Open settings folder</button>
  </Field>
{/if}

<style>
  .row {
    display: flex;
    gap: 8px;
  }
  .danger {
    background: var(--err) !important;
    color: #fff !important;
  }
</style>
