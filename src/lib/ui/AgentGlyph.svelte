<script lang="ts">
  import { Check, X } from '@lucide/svelte';
  import { beat } from '../stores/beat.svelte';
  import type { AgentSession } from '../types';

  let { status, size = 16 }: { status: AgentSession['status']; size?: number } = $props();

  // Both loops step on the shared 8 fps beat: see stores/beat.
  const looping = $derived(status === 'working' || status === 'waiting');
  $effect(() => (looping ? beat.use() : undefined));
  const spin = $derived(status === 'working' ? (beat.n % 12) * 30 : 0);
  /** Waiting pulse: a ring that grows and fades over 11 beats (~1.4 s). */
  const pulse = $derived(status === 'waiting' ? (beat.n % 11) / 10 : 0);
</script>

<span class="glyph {status}" style="width:{size}px;height:{size}px" aria-label={status}>
  {#if status === 'done' || status === 'ended'}
    <Check size={size * 0.8} strokeWidth={2.6} />
  {:else if status === 'error'}
    <X size={size * 0.8} strokeWidth={2.6} />
  {:else}
    <i style="--spin:{spin}deg;--p:{pulse}"></i>
  {/if}
</span>

<style>
  .glyph {
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
  .working i {
    border: 2px solid var(--accent-soft);
    border-top-color: var(--accent);
    transform: rotate(var(--spin));
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
