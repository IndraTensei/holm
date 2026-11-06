<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    label,
    onclick,
    active = false,
    size = 32,
    tone = 'plain',
    disabled = false,
    children,
  }: {
    label: string;
    onclick?: (e: MouseEvent) => void;
    active?: boolean;
    size?: number;
    tone?: 'plain' | 'solid' | 'ok' | 'err';
    disabled?: boolean;
    children: Snippet;
  } = $props();
</script>

<button
  class="btn {tone}"
  class:active
  {disabled}
  aria-label={label}
  title={label}
  style="width:{size}px;height:{size}px"
  onclick={(e) => {
    e.stopPropagation();
    onclick?.(e);
  }}
>
  {@render children()}
</button>

<style>
  .btn {
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 999px;
    color: var(--ink-2);
    transition: background 0.18s, color 0.18s, transform 0.12s;
  }
  .btn:hover {
    background: var(--fill-1);
    color: var(--ink);
  }
  .btn:active {
    transform: scale(0.92);
  }
  .btn:disabled {
    opacity: 0.35;
    pointer-events: none;
  }
  .active {
    color: var(--accent);
  }
  .solid {
    background: var(--ink);
    color: var(--island);
  }
  .solid:hover {
    background: var(--ink);
    color: var(--island);
    transform: scale(1.04);
  }
  .ok {
    background: var(--ok);
    color: #04140b;
  }
  .err {
    background: var(--fill-2);
    color: var(--ink);
  }
</style>
