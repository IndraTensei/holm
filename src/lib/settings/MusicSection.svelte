<script lang="ts">
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import Choice from './Choice.svelte';
  import { settings } from '../stores/settings.svelte';
  import { pear } from '../stores/pear.svelte';

  const s = $derived(settings.v);
  const statusText = $derived(
    {
      off: 'Not checked yet',
      connecting: 'Connecting…',
      'needs-auth': 'Found Pear. Connect to allow access.',
      denied: 'Pear refused. Allow it in Pear and try again.',
      connected: pear.song ? `Connected, playing ${pear.song.title}` : 'Connected',
      unreachable: 'Pear is not answering. Is it open, with the API Server plugin on?',
    }[pear.status],
  );
</script>

<section>
  <h2 class="display">Music</h2>
  <p class="lead">
    Works with no setup: Spotify, Apple Music, TIDAL, Deezer, Amazon Music, VLC, browsers (YouTube, SoundCloud, Spotify web) and anything else that shows in the Windows media card. The volume slider sets the playing app's own volume, not the whole PC's. Pear can also connect for likes, shuffle and repeat.
  </p>

  <Field label="Source">
    <Choice
      label="Source"
      value={s.musicSource}
      options={[
        { value: 'auto', label: 'Automatic' },
        { value: 'pear', label: 'Pear only' },
        { value: 'system', label: 'Windows only' },
      ]}
      onchange={(v) => settings.set({ musicSource: v })}
    />
  </Field>

  {#if s.musicSource !== 'system'}
    <div class="pear">
      <div class="pear-head">
        <span class="status" class:ok={pear.status === 'connected'} class:warn={pear.status === 'needs-auth' || pear.status === 'denied'}></span>
        <div>
          <div class="p-title">Pear Desktop</div>
          <div class="p-text">{statusText}</div>
        </div>
        {#if pear.status === 'connected'}
          <button class="ghost" onclick={() => { pear.stop(); void settings.set({ pearToken: '' }); }}>Disconnect</button>
        {:else}
          <button class="primary" onclick={() => pear.authorize()}>Connect Pear</button>
        {/if}
      </div>
      <ol class="steps">
        <li>In Pear, open Plugins and turn on <b>API Server</b>.</li>
        <li>Click Connect Pear here, then Allow in the dialog Pear shows.</li>
        <li>Likes, dislikes, Pear's own volume, shuffle and repeat now work from the island.</li>
      </ol>
      <div class="addr">
        <label>Address <input value={s.pearHost} onchange={(e) => settings.set({ pearHost: (e.target as HTMLInputElement).value.trim() || '127.0.0.1' })} /></label>
        <label>Port <input class="num" type="number" value={s.pearPort} onchange={(e) => settings.set({ pearPort: Number((e.target as HTMLInputElement).value) || 26538 })} /></label>
      </div>
    </div>
  {/if}

  <Field label="Synced lyrics" hint="From LRCLIB, a free lyrics library. Only the title, artist and length are sent.">
    <Toggle label="Synced lyrics" checked={s.lyrics} onchange={(v) => settings.set({ lyrics: v })} />
  </Field>
  <Field label="Show new tracks" hint="The island grows for a moment when the song changes.">
    <Toggle label="Show new tracks" checked={s.trackToasts} onchange={(v) => settings.set({ trackToasts: v })} />
  </Field>
  <Field label="Show volume changes" hint="A volume bar in the island when you press the volume keys.">
    <Toggle label="Show volume changes" checked={s.volumeOsd} onchange={(v) => settings.set({ volumeOsd: v })} />
  </Field>
</section>

<style>
  .pear {
    margin: 6px 0 10px;
    padding: 14px 16px;
    border-radius: 14px;
    background: var(--fill-1);
  }
  .pear-head {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .pear-head > div {
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
  .p-title {
    font-weight: 600;
  }
  .p-text {
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .steps {
    margin: 12px 0 10px;
    padding-left: 20px;
    font-size: var(--t-sm);
    line-height: 1.6;
    color: var(--ink-2);
  }
  .steps b {
    color: var(--ink);
  }
  .addr {
    display: flex;
    gap: 14px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .addr input {
    margin-left: 6px;
    width: 120px;
    height: 28px;
    padding: 0 8px;
    border: 0;
    border-radius: 7px;
    background: var(--fill-1);
  }
  .addr input[type='number'] {
    width: 80px;
  }
</style>
