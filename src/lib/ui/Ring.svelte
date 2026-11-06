<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    value,
    size = 20,
    stroke = 2.5,
    color = '',
    track = 'var(--fill-2)',
    children,
  }: { value: number; size?: number; stroke?: number; color?: string; track?: string; children?: Snippet } = $props();

  const r = $derived((size - stroke) / 2);
  const c = $derived(2 * Math.PI * r);
  const v = $derived(Math.max(0, Math.min(100, value)));
  const tone = $derived(color || (v >= 90 ? 'var(--err)' : v >= 70 ? 'var(--warn)' : 'var(--accent)'));
</script>

<div class="ring" style="width:{size}px;height:{size}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle cx={size / 2} cy={size / 2} {r} fill="none" stroke={track} stroke-width={stroke} />
    <circle
      cx={size / 2}
      cy={size / 2}
      {r}
      fill="none"
      stroke={tone}
      stroke-width={stroke}
      stroke-linecap="round"
      stroke-dasharray={c}
      stroke-dashoffset={c * (1 - v / 100)}
      transform="rotate(-90 {size / 2} {size / 2})"
    />
  </svg>
  {#if children}<div class="inner">{@render children()}</div>{/if}
</div>

<style>
  .ring {
    position: relative;
    flex: none;
    display: grid;
    place-items: center;
  }
  svg {
    position: absolute;
    inset: 0;
  }
  circle {
    transition: stroke-dashoffset 0.6s var(--ease-out), stroke 0.3s;
  }
  .inner {
    position: relative;
    display: grid;
    place-items: center;
  }
</style>
