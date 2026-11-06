<script lang="ts">
  // A moment of news: the island grows, says one thing, and settles back.
  import { Check, BatteryCharging, Volume2, VolumeX, FolderDown, Timer, MessageCircleQuestion, Plug } from '@lucide/svelte';
  import Art from '../ui/Art.svelte';
  import Eq from '../ui/Eq.svelte';
  import type { Peek } from '../stores/activity.svelte';
  import { duration } from '../format';

  let { peek }: { peek: Peek } = $props();
  const d = $derived(peek.data as Record<string, any>);
</script>

{#if peek.kind === 'done'}
  <div class="peek wide">
    <span class="badge ok"><Check size={16} strokeWidth={2.8} /></span>
    <div class="text">
      <div class="title display">{d.project} is done</div>
      <div class="sub">
        {duration(d.durationMs)}{#if d.edited?.length}, {d.edited.length} {d.edited.length === 1 ? 'file' : 'files'} changed{/if}
      </div>
      {#if d.message}<p class="msg">{d.message}</p>{/if}
    </div>
  </div>
{:else if peek.kind === 'attention'}
  <div class="peek wide">
    <span class="badge warn"><MessageCircleQuestion size={16} strokeWidth={2.4} /></span>
    <div class="text">
      <div class="title display">{d.project} needs you</div>
      {#if d.message}<p class="msg">{d.message}</p>{/if}
    </div>
  </div>
{:else if peek.kind === 'track'}
  <div class="peek">
    <Art src={d.art} size={42} radius={10} />
    <div class="text">
      <div class="title display one">{d.title}</div>
      <div class="sub one">{d.artist}</div>
    </div>
    <Eq playing bars={4} height={16} />
  </div>
{:else if peek.kind === 'volume'}
  <div class="peek slim">
    {#if d.muted || d.level === 0}<VolumeX size={17} />{:else}<Volume2 size={17} />{/if}
    <div class="bar"><span style="width:{(d.muted ? 0 : d.level) * 100}%"></span></div>
    <span class="num pct">{d.muted ? 'Muted' : Math.round(d.level * 100)}</span>
  </div>
{:else if peek.kind === 'charging'}
  <div class="peek slim">
    <span class="charge"><BatteryCharging size={18} strokeWidth={2} /></span>
    <span class="label">Charging</span>
    <span class="num pct big">{d.percent}%</span>
  </div>
{:else if peek.kind === 'timer'}
  <div class="peek">
    <span class="badge accent"><Timer size={16} strokeWidth={2.4} /></span>
    <div class="text">
      <div class="title display">{d.phase === 'focus' ? 'Focus session done' : 'Break is over'}</div>
      <div class="sub">{d.phase === 'focus' ? 'Take a short break.' : 'Ready for the next one?'}</div>
    </div>
  </div>
{:else if peek.kind === 'shelf'}
  <div class="peek slim">
    <FolderDown size={17} />
    <span class="label">{d.count} {d.count === 1 ? 'file' : 'files'} added to the shelf</span>
  </div>
{:else if peek.kind === 'pear'}
  <div class="peek slim">
    <Plug size={17} />
    <span class="label">{d.text}</span>
  </div>
{/if}

<style>
  .peek {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px 12px 12px;
    min-width: 320px;
    max-width: 420px;
  }
  .wide {
    align-items: flex-start;
    width: 420px;
    padding: 14px 18px 15px 14px;
  }
  .slim {
    height: 44px;
    padding: 0 18px;
    min-width: 260px;
    gap: 10px;
    color: var(--ink-2);
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: var(--t-lg);
    font-weight: 650;
    letter-spacing: -0.015em;
    line-height: 1.2;
  }
  .sub {
    margin-top: 2px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .one {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .msg {
    margin: 8px 0 0;
    font-size: var(--t-sm);
    line-height: 1.45;
    color: var(--ink-2);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .badge {
    flex: none;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
  }
  .badge.ok {
    background: color-mix(in srgb, var(--ok) 18%, transparent);
    color: var(--ok);
  }
  .badge.warn {
    background: color-mix(in srgb, var(--warn) 18%, transparent);
    color: var(--warn);
  }
  .badge.accent {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .bar {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--fill-2);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--ink);
    border-radius: 3px;
    transition: width 0.12s linear;
  }
  .pct {
    min-width: 34px;
    text-align: right;
    font-size: var(--t-sm);
    font-weight: 600;
    color: var(--ink);
  }
  .pct.big {
    margin-left: auto;
    color: var(--ok);
  }
  .label {
    font-size: var(--t-sm);
    font-weight: 500;
    color: var(--ink);
  }
  .charge {
    color: var(--ok);
    display: grid;
  }
</style>
