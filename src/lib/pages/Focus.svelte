<script lang="ts">
  import { Play, Pause, RotateCcw, Coffee } from '@lucide/svelte';
  import { focus } from '../stores/widgets.svelte';
  import { settings } from '../stores/settings.svelte';
  import { clock } from '../stores/clock.svelte';
  import { clockTime } from '../format';

  $effect(() => (focus.running ? clock.second() : undefined));

  const presets = [15, 25, 50];
  const remaining = $derived(focus.remaining(clock.now));
  const progress = $derived(focus.progress(clock.now));
  const isBreak = $derived(focus.phase === 'break' || (focus.phase === 'paused' && focus.pausedPhase === 'break'));
  const size = 148;
  const r = 66;
  const c = 2 * Math.PI * r;
</script>

<div class="focus">
  <div class="dial" style="width:{size}px;height:{size}px">
    <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
      <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke="var(--fill-2)" stroke-width="6" />
      <circle
        cx={size / 2}
        cy={size / 2}
        {r}
        fill="none"
        stroke={isBreak ? 'var(--ok)' : 'var(--accent)'}
        stroke-width="6"
        stroke-linecap="round"
        stroke-dasharray={c}
        stroke-dashoffset={c * (1 - progress)}
        transform="rotate(-90 {size / 2} {size / 2})"
      />
    </svg>
    <div class="readout">
      <span class="time display num">{clockTime(remaining / 1000)}</span>
      <span class="phase">{focus.phase === 'idle' ? 'Ready' : focus.phase === 'paused' ? 'Paused' : isBreak ? 'Break' : 'Focus'}</span>
    </div>
  </div>

  <div class="side">
    <div class="actions">
      {#if focus.phase === 'idle'}
        <button class="primary" onclick={() => focus.start('focus')}><Play size={15} fill="currentColor" /> Start focus</button>
        <button onclick={() => focus.start('break')}><Coffee size={15} /> Break</button>
      {:else if focus.phase === 'paused'}
        <button class="primary" onclick={() => focus.resume()}><Play size={15} fill="currentColor" /> Resume</button>
        <button onclick={() => focus.reset()}><RotateCcw size={15} /> Reset</button>
      {:else}
        <button class="primary" onclick={() => focus.pause()}><Pause size={15} fill="currentColor" /> Pause</button>
        <button onclick={() => focus.reset()}><RotateCcw size={15} /> Reset</button>
      {/if}
    </div>
    <div class="presets" role="radiogroup" aria-label="Focus length">
      {#each presets as m}
        <button
          role="radio"
          aria-checked={settings.v.focusMinutes === m}
          class:on={settings.v.focusMinutes === m}
          onclick={() => {
            void settings.set({ focusMinutes: m });
            if (focus.phase === 'idle') return;
            if (!isBreak) focus.start('focus', m);
          }}
        ><span class="num">{m}</span> min</button>
      {/each}
    </div>
    <p class="today">
      {#if focus.sessionsToday === 0}No sessions finished today yet.{:else}{focus.sessionsToday} {focus.sessionsToday === 1 ? 'session' : 'sessions'} finished today.{/if}
      The island shows the countdown while you work.
    </p>
  </div>
</div>

<style>
  .focus {
    display: flex;
    align-items: center;
    gap: 28px;
    padding: 6px 8px 4px;
  }
  .dial {
    position: relative;
    flex: none;
  }
  svg {
    position: absolute;
    inset: 0;
  }
  circle {
    transition: stroke-dashoffset 0.3s linear, stroke 0.3s;
  }
  .readout {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }
  .time {
    font-size: 34px;
    font-weight: 650;
    letter-spacing: -0.03em;
  }
  .phase {
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .side {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .actions button {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 36px;
    padding: 0 16px;
    border-radius: 999px;
    background: var(--fill-1);
    font-weight: 600;
    font-size: var(--t-sm);
    transition: background 0.15s, transform 0.12s;
  }
  .actions button:hover {
    background: var(--fill-2);
  }
  .actions button:active {
    transform: scale(0.96);
  }
  .actions .primary {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .actions .primary:hover {
    background: var(--accent);
    filter: brightness(1.06);
  }
  .presets {
    display: flex;
    gap: 4px;
  }
  .presets button {
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    font-size: var(--t-xs);
    font-weight: 600;
    color: var(--ink-3);
  }
  .presets button.on {
    background: var(--fill-2);
    color: var(--ink);
  }
  .today {
    margin: 0;
    font-size: var(--t-xs);
    line-height: 1.5;
    color: var(--ink-3);
    max-width: 40ch;
  }
</style>
