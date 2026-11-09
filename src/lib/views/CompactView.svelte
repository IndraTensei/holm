<script lang="ts">
  // The resting island. Shows the most important live activity, iOS style:
  // something on the left, something on the right, as little as possible in between.
  import { Timer, Coffee, BatteryCharging } from '@lucide/svelte';
  import Art from '../ui/Art.svelte';
  import Eq from '../ui/Eq.svelte';
  import Ring from '../ui/Ring.svelte';
  import AgentGlyph from '../ui/AgentGlyph.svelte';
  import Marquee from '../ui/Marquee.svelte';
  import { media } from '../stores/media.svelte';
  import { agents } from '../stores/agents.svelte';
  import { focus } from '../stores/widgets.svelte';
  import { system } from '../stores/system.svelte';
  import { lyrics } from '../stores/lyrics.svelte';
  import { settings } from '../stores/settings.svelte';
  import { clock } from '../stores/clock.svelte';
  import { clockTime } from '../format';

  import type { Live } from '../types';
  let { primary, line = false }: { primary: Live; line?: boolean } = $props();

  const now = $derived(media.now);
  const agent = $derived(agents.focus);
  const s = $derived(settings.v);

  const time = $derived.by(() => {
    const d = new Date(clock.now);
    return d.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit', hour12: !s.clock24h }).replace(/\s?[AP]M$/i, '');
  });

  // The sung line, moved on by one timer set for the next line's start rather than a fast
  // clock: about one wake-up per lyric line. The effect re-runs when the player reports a new
  // position (a seek, pause or skip), because media.position reads that state.
  const showLyrics = $derived(primary === 'music' && s.compactTitle === 'lyrics' && lyrics.status === 'synced');
  let lyricIndex = $state(-1);
  $effect(() => {
    if (!showLyrics || !now) {
      lyricIndex = -1;
      return;
    }
    const lines = lyrics.lines;
    const playing = now.playing;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const follow = () => {
      const pos = media.position(Date.now());
      const i = lyrics.index(pos);
      lyricIndex = i;
      const next = lines[i + 1];
      // index() switches 0.25 s early, so the next line is due at next.time - 0.25.
      if (playing && next) timer = setTimeout(follow, Math.max(50, (next.time - 0.25 - pos) * 1000 + 20));
    };
    follow();
    return () => clearTimeout(timer);
  });
  const lyricLine = $derived(showLyrics && lyricIndex >= 0 ? (lyrics.lines[lyricIndex]?.text ?? '') : '');

  const middle = $derived(
    primary === 'music' && s.compactTitle !== 'none' ? lyricLine || now?.title || '' : '',
  );

  const lineTone = $derived(
    agent?.status === 'waiting'
      ? 'var(--warn)'
      : primary === 'agent'
        ? 'var(--accent)'
        : primary === 'music' && now?.playing
          ? 'var(--accent)'
          : 'var(--ink-3)',
  );
</script>

{#if line}
  <div class="line" style="--tone:{lineTone}"><span></span></div>
{:else if primary === 'music' && now}
  <div class="row">
    <Art src={now.art} size={22} radius={6} />
    {#if middle}
      <Marquee text={middle} once={showLyrics} class={showLyrics ? 'mid lyric' : 'mid'} />
    {:else}
      <span class="gap"></span>
    {/if}
    <Eq playing={now.playing} bars={4} height={13} />
  </div>
{:else if primary === 'agent' && agent}
  <div class="row">
    <AgentGlyph status={agent.status} size={15} />
    <span class="agent">
      {#if agent.status === 'waiting'}
        <b>{agent.project}</b> needs you
      {:else if agent.activity && agent.activity.target}
        <b>{agent.activity.verb}</b> {agent.activity.target}
      {:else}
        <b>{agent.project}</b> {agent.activity?.verb.toLowerCase() ?? 'working'}
      {/if}
    </span>
    <span class="num elapsed">{clockTime((clock.now - agent.turnStartedAt) / 1000)}</span>
  </div>
{:else if primary === 'timer'}
  <div class="row">
    <Ring value={focus.progress(clock.now) * 100} size={18} stroke={2.4} color={focus.pausedPhase === 'break' || focus.phase === 'break' ? 'var(--ok)' : 'var(--accent)'}>
      {#if focus.phase === 'break'}<Coffee size={9} strokeWidth={2.4} />{:else}<Timer size={9} strokeWidth={2.4} />{/if}
    </Ring>
    <span class="gap"></span>
    <span class="num timer display" class:paused={focus.phase === 'paused'}>{clockTime(focus.remaining(clock.now) / 1000)}</span>
  </div>
{:else}
  <div class="row idle">
    {#if s.showQuotaRing && system.topQuota}
      <span class="quota" title="{system.topQuota.provider === 'claude' ? 'Claude' : 'Codex'} {system.topQuota.label.toLowerCase()} limit">
        <Ring value={system.topQuota.percent} size={16} stroke={2.6} />
        <span class="num">{Math.round(system.topQuota.percent)}%</span>
      </span>
    {:else if s.showBattery && system.battery && (system.battery.charging || system.battery.percent <= 20)}
      <span class="battery" class:low={!system.battery.plugged && system.battery.percent <= 20}>
        {#if system.battery.charging}<BatteryCharging size={15} strokeWidth={2} />{/if}
        <span class="num">{system.battery.percent}</span>
      </span>
    {:else}
      <span class="dot"></span>
    {/if}
    <span class="gap"></span>
    {#if s.showClock}<span class="num clock display">{time}</span>{/if}
  </div>
{/if}

<style>
  .row {
    height: var(--h);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 calc(var(--h) * 0.3);
    min-width: 176px;
    white-space: nowrap;
  }
  .gap {
    flex: 1;
    min-width: 28px;
  }
  .row :global(.mid) {
    flex: 1;
    max-width: 220px;
    font-size: var(--t-sm);
    font-weight: 500;
    color: var(--ink-2);
  }
  /* A song with synced lyrics keeps one width for all its lines, so the island doesn't
     resize (and re-shape its window) every few seconds; longer lines slide. */
  .row :global(.lyric) {
    flex: none;
    width: 240px;
    max-width: none;
  }
  .agent {
    max-width: 260px;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .agent b {
    font-weight: 600;
    color: var(--ink);
  }
  .elapsed {
    margin-left: auto;
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .timer {
    font-size: var(--t-md);
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--accent);
  }
  .timer.paused {
    color: var(--ink-3);
  }
  .clock {
    font-size: var(--t-md);
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--ink);
  }
  .idle {
    min-width: 150px;
  }
  .quota {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: var(--t-xs);
    font-weight: 600;
    color: var(--ink-2);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ink-4);
  }
  .battery {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: var(--t-sm);
    font-weight: 600;
    color: var(--ok);
  }
  .battery.low {
    color: var(--err);
  }
  .line {
    width: 132px;
    height: 10px;
    display: grid;
    place-items: center;
  }
  .line span {
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--tone);
    opacity: 0.85;
  }
</style>
