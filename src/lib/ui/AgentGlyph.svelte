<script lang="ts">
  import { Check, X } from '@lucide/svelte';
  import { beat } from '../stores/beat.svelte';
  import type { AgentSession } from '../types';

  let { status, size = 16 }: { status: AgentSession['status']; size?: number } = $props();

  // Both loops step on the shared 8 fps beat: see stores/beat.
  const looping = $derived(status === 'working' || status === 'waiting');
  $effect(() => (looping ? beat.use() : undefined));
  // Working: eight spokes with a fading trail, the way system spinners do it. A spinner like this
  // is discrete by design, so one step per beat reads as smooth motion (a rotating arc stepped at
  // the same rate looked choppy) at the same low cost.
  const SPOKES = 8;
  const head = $derived(status === 'working' ? beat.n % SPOKES : 0);
  const spokes = Array.from({ length: SPOKES }, (_, i) => {
    const a = (i / SPOKES) * Math.PI * 2 - Math.PI / 2;
    const [c, sn] = [Math.cos(a), Math.sin(a)];
    return { x1: 8 + c * 3.4, y1: 8 + sn * 3.4, x2: 8 + c * 6.6, y2: 8 + sn * 6.6 };
  });
  const fade = (i: number) => 0.18 + 0.82 * Math.pow(1 - ((head - i + SPOKES) % SPOKES) / SPOKES, 2);
  /** Waiting pulse: a ring that grows and fades over 11 beats (~1.4 s). */
  const pulse = $derived(status === 'waiting' ? (beat.n % 11) / 10 : 0);
</script>

<span class="glyph {status}" style="width:{size}px;height:{size}px" aria-label={status}>
  {#if status === 'done' || status === 'ended'}
    <Check size={size * 0.8} strokeWidth={2.6} />
  {:else if status === 'error'}
    <X size={size * 0.8} strokeWidth={2.6} />
  {:else if status === 'working'}
    <svg viewBox="0 0 16 16" width={size} height={size} aria-hidden="true">
      {#each spokes as l, i}
        <line {...l} style="opacity:{fade(i)}" />
      {/each}
    </svg>
  {:else}
    <i style="--p:{pulse}"></i>
  {/if}
</span>

<style>
  /* Its own compositing layer: each beat repaints these pixels, not the whole island. */
  .glyph {
    will-change: transform;
    position: relative;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 50%;
  }
  i {
    position: absolute;
    inset: 0;
    border-radius: 50%;
  }
  svg line {
    stroke: var(--accent);
    stroke-width: 1.7;
    stroke-linecap: round;
  }
  .waiting i {
    inset: 22%;
    background: var(--warn);
  }
  .waiting i::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 2px solid var(--warn);
    transform: scale(calc(1 + var(--p) * 1.6));
    opacity: calc(0.6 * (1 - var(--p)));
  }
  .idle i {
    inset: 30%;
    background: var(--ink-3);
  }
  .done,
  .ended {
    color: var(--ok);
  }
  .error {
    color: var(--err);
  }
</style>
