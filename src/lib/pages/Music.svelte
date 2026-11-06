<script lang="ts">
  import {
    Play, Pause, SkipBack, SkipForward, Shuffle, Repeat, Repeat1, Heart, ThumbsDown, Volume2, VolumeX, ChevronDown, Plug,
  } from '@lucide/svelte';
  import Art from '../ui/Art.svelte';
  import Btn from '../ui/Btn.svelte';
  import Scrub from '../ui/Scrub.svelte';
  import { media } from '../stores/media.svelte';
  import { pear } from '../stores/pear.svelte';
  import { lyrics } from '../stores/lyrics.svelte';
  import { system } from '../stores/system.svelte';
  import { settings } from '../stores/settings.svelte';
  import { clock } from '../stores/clock.svelte';
  import { clockTime } from '../format';
  import { call, send } from '../bridge';

  const now = $derived(media.now);
  $effect(() => (now?.playing ? clock.fast() : undefined));

  const position = $derived(media.position(clock.now));
  const line = $derived(lyrics.status === 'synced' ? lyrics.index(position) : -1);
  let picking = $state(false);
  let lyricBox = $state<HTMLDivElement>();

  // Keep the sung line in the middle of the lyrics box.
  $effect(() => {
    if (line < 0 || !lyricBox) return;
    const el = lyricBox.querySelector<HTMLElement>(`[data-i="${line}"]`);
    if (el) lyricBox.scrollTo({ top: el.offsetTop - lyricBox.clientHeight / 2 + el.clientHeight / 2, behavior: 'smooth' });
  });

  // Any player's own volume through the Windows mixer (Spotify, a browser, VLC…); the PC volume
  // only when the player's sound cannot be found there. Pear uses its own API.
  let appVolume = $state<{ level: number; muted: boolean } | null>(null);
  const appSession = $derived(now && now.source !== 'pear' ? now.sessionId : null);
  $effect(() => {
    const session = appSession;
    appVolume = null;
    if (session) void call<{ level: number; muted: boolean } | null>('app_volume_get', { session }).then((v) => {
      if (session === appSession) appVolume = v;
    }).catch(() => {});
  });
  const volume = $derived(
    now?.source === 'pear'
      ? (now.volume ?? 0) / 100
      : appVolume
        ? appVolume.muted ? 0 : appVolume.level
        : system.volume?.muted ? 0 : (system.volume?.level ?? 0),
  );
  const volumeLabel = $derived(now?.source === 'pear' || appVolume ? `${now?.app ?? 'Player'} volume` : 'PC volume');
  function setVolume(v: number) {
    if (now?.source === 'pear') void pear.setVolume(v * 100);
    else if (appVolume && appSession) {
      appVolume = { level: v, muted: false };
      send('app_volume_set', { session: appSession, level: v });
    } else system.setVolume(v);
  }
  function toggleMute() {
    if (now?.source === 'pear') void pear.toggleMute();
    else if (appVolume && appSession) {
      appVolume = { ...appVolume, muted: !appVolume.muted };
      send('app_volume_mute', { session: appSession, muted: appVolume.muted });
    } else system.toggleMute();
  }
  // Only while Pear is the player: Spotify and the rest need no setup.
  const pearHint = $derived(now?.app === 'Pear' && settings.v.musicSource !== 'system' && (pear.status === 'needs-auth' || pear.status === 'denied'));
</script>

{#if now}
  <div class="music">
    <div class="top">
      <div class="cover">
        <div class="glow" style="background-image:url({now.art ?? ''})"></div>
        <Art src={now.art} size={124} radius={18} alt="" />
      </div>
      <div class="info">
        <div class="source">
          <button class="app" onclick={() => (picking = !picking)} disabled={media.options.length < 2} aria-expanded={picking}>
            {now.app}
            {#if media.options.length > 1}<ChevronDown size={13} />{/if}
          </button>
          {#if picking}
            <div class="menu" role="menu">
              {#each media.options as o (o.sessionId)}
                <button
                  role="menuitem"
                  class:on={o.sessionId === now.sessionId}
                  onclick={() => {
                    media.pinned = o.sessionId;
                    picking = false;
                  }}
                >
                  <span class="m-app">{o.app}</span>
                  <span class="m-title">{o.title}</span>
                </button>
              {/each}
              {#if media.pinned}
                <button role="menuitem" class="follow" onclick={() => { media.pinned = null; picking = false; }}>Follow what plays</button>
              {/if}
            </div>
          {/if}
        </div>
        <h2 class="title display">{now.title}</h2>
        <div class="artist">{now.artist}{#if now.album && now.album !== now.title}<span class="album">{now.album}</span>{/if}</div>

        {#if now.duration > 0}
          <div class="time">
            <Scrub value={position} max={now.duration} onseek={media.seek} disabled={!now.canSeek} />
            <div class="stamps num"><span>{clockTime(position)}</span><span>-{clockTime(now.duration - position)}</span></div>
          </div>
        {/if}

        <div class="controls">
          <Btn label="Shuffle" active={now.shuffle === true} onclick={media.shuffle} disabled={now.shuffle === null} size={30}>
            <Shuffle size={15} />
          </Btn>
          <Btn label="Previous" onclick={media.prev} disabled={!now.canPrev} size={36}><SkipBack size={19} fill="currentColor" /></Btn>
          <Btn label={now.playing ? 'Pause' : 'Play'} tone="solid" onclick={media.toggle} size={44}>
            {#if now.playing}<Pause size={20} fill="currentColor" />{:else}<Play size={20} fill="currentColor" style="margin-left:2px" />{/if}
          </Btn>
          <Btn label="Next" onclick={media.next} disabled={!now.canNext} size={36}><SkipForward size={19} fill="currentColor" /></Btn>
          <Btn label="Repeat" active={now.repeat === 'list' || now.repeat === 'track'} onclick={media.repeat} disabled={now.repeat === null} size={30}>
            {#if now.repeat === 'track'}<Repeat1 size={15} />{:else}<Repeat size={15} />{/if}
          </Btn>
          <span class="spacer"></span>
          {#if now.source === 'pear'}
            <Btn label="Dislike" active={now.like === 'DISLIKE'} onclick={media.dislike} size={30}>
              <ThumbsDown size={15} fill={now.like === 'DISLIKE' ? 'currentColor' : 'none'} />
            </Btn>
            <Btn label={now.like === 'LIKE' ? 'Remove like' : 'Like'} active={now.like === 'LIKE'} onclick={media.like} size={30}>
              <Heart size={15} fill={now.like === 'LIKE' ? 'currentColor' : 'none'} />
            </Btn>
          {/if}
          <div class="volume">
            <button class="mute" aria-label="Mute {volumeLabel}" title={volumeLabel} onclick={toggleMute}>
              {#if volume === 0}<VolumeX size={15} />{:else}<Volume2 size={15} />{/if}
            </button>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={volume}
              aria-label={volumeLabel}
              title={volumeLabel}
              style="--v:{volume * 100}%"
              oninput={(e) => setVolume(Number((e.target as HTMLInputElement).value))}
            />
          </div>
        </div>
      </div>
    </div>

    {#if pearHint}
      <div class="hint">
        <Plug size={15} />
        <span>{pear.status === 'denied' ? 'Pear refused the connection. Allow it in Pear, then try again.' : 'Connect Pear for likes, Pear volume and exact progress.'}</span>
        <button onclick={() => pear.authorize()}>Connect Pear</button>
      </div>
    {/if}

    {#if settings.v.lyrics && (lyrics.status === 'synced' || lyrics.status === 'plain')}
      <div class="lyrics" bind:this={lyricBox} class:plain={lyrics.status === 'plain'}>
        {#if lyrics.status === 'synced'}
          {#each lyrics.lines as l, i}
            <button
              data-i={i}
              class="l display"
              class:past={i < line}
              class:on={i === line}
              disabled={!now.canSeek}
              onclick={() => media.seek(l.time)}
            >{l.text || '♪'}</button>
          {/each}
        {:else}
          <p>{lyrics.plain}</p>
        {/if}
      </div>
    {:else if settings.v.lyrics && lyrics.status === 'loading'}
      <div class="lyrics-note">Looking for lyrics…</div>
    {/if}
  </div>
{:else}
  <div class="empty">
    <p class="display">Nothing is playing</p>
    <span>Play something in Spotify, Pear, a browser or any player that shows in the Windows media card.</span>

  </div>
{/if}

<style>
  .music {
    padding: 4px 4px 2px;
  }
  .top {
    display: flex;
    gap: 20px;
  }
  .cover {
    position: relative;
    flex: none;
    width: 124px;
    height: 124px;
  }
  .glow {
    position: absolute;
    inset: 10px;
    background-size: cover;
    background-position: center;
    filter: blur(26px) saturate(1.5);
    opacity: 0.55;
    transform: translateY(8px) scale(1.05);
  }
  .cover :global(.art) {
    position: relative;
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .source {
    position: relative;
  }
  .app {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: var(--t-xs);
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 0.01em;
  }
  .app:disabled {
    cursor: default;
  }
  .menu {
    position: absolute;
    z-index: 5;
    top: 20px;
    left: -6px;
    min-width: 240px;
    padding: 5px;
    border-radius: 14px;
    background: color-mix(in srgb, var(--island) 88%, var(--ink) 12%);
    box-shadow: 0 12px 32px -8px rgba(0, 0, 0, 0.5), inset 0 0 0 0.5px var(--line);
  }
  .menu button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 7px 10px;
    border-radius: 9px;
    text-align: left;
  }
  .menu button:hover,
  .menu button.on {
    background: var(--fill-1);
  }
  .m-app {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .m-title {
    font-size: var(--t-sm);
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .follow {
    font-size: var(--t-sm);
    color: var(--accent);
  }
  .title {
    margin: 4px 0 0;
    font-size: var(--t-xl);
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.15;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .artist {
    margin-top: 3px;
    font-size: var(--t-md);
    color: var(--ink-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .album {
    color: var(--ink-3);
    margin-left: 8px;
  }
  .time {
    margin-top: 10px;
  }
  .stamps {
    display: flex;
    justify-content: space-between;
    margin-top: 1px;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .controls {
    margin-top: 6px;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .spacer {
    flex: 1;
  }
  .volume {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 104px;
    color: var(--ink-2);
  }
  .mute {
    display: grid;
  }
  input[type='range'] {
    -webkit-appearance: none;
    appearance: none;
    flex: 1;
    min-width: 0;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: linear-gradient(90deg, var(--ink-2) var(--v), var(--fill-2) var(--v));
    cursor: pointer;
  }
  input[type='range']::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--ink);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
  }
  .hint {
    margin-top: 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px 9px 12px;
    border-radius: 12px;
    background: var(--accent-soft);
    color: var(--ink);
    font-size: var(--t-sm);
  }
  .hint span {
    flex: 1;
  }
  .hint button,
  .lyrics {
    position: relative;
    margin-top: 16px;
    height: 112px;
    overflow-y: auto;
    scrollbar-width: none;
    mask-image: linear-gradient(transparent, #000 26%, #000 74%, transparent);
    padding: 34px 0;
  }
  .lyrics::-webkit-scrollbar {
    display: none;
  }
  .l {
    display: block;
    width: 100%;
    text-align: left;
    padding: 2px 0;
    font-size: var(--t-lg);
    font-weight: 600;
    letter-spacing: -0.01em;
    line-height: 1.35;
    color: var(--ink-4);
    transition: color 0.35s, transform 0.35s var(--ease-out);
    transform-origin: left center;
  }
  .l.past {
    color: var(--ink-3);
  }
  .l.on {
    color: var(--ink);
    transform: scale(1.02);
  }
  .l:disabled {
    cursor: default;
  }
  .plain p {
    margin: 0;
    white-space: pre-line;
    font-size: var(--t-md);
    line-height: 1.6;
    color: var(--ink-2);
  }
  .lyrics-note {
    margin-top: 14px;
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .empty {
    padding: 22px 8px 18px;
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
    max-width: 52ch;
    line-height: 1.5;
  }
</style>
