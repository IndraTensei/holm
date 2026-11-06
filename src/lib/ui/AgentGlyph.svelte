<script lang="ts">
  import { Check, X } from '@lucide/svelte';
  import type { AgentSession } from '../types';

  let { status, size = 16 }: { status: AgentSession['status']; size?: number } = $props();
</script>

<span class="glyph {status}" style="width:{size}px;height:{size}px" aria-label={status}>
  {#if status === 'done' || status === 'ended'}
    <Check size={size * 0.8} strokeWidth={2.6} />
  {:else if status === 'error'}
    <X size={size * 0.8} strokeWidth={2.6} />
  {:else}
    <i></i>
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
    animation: spin 0.9s linear infinite;
  }
  .waiting i {
    inset: 22%;
    background: var(--warn);
    box-shadow: 0 0 0 0 var(--warn);
    animation: pulse 1.4s ease-out infinite;
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
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  @keyframes pulse {
    0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--warn) 60%, transparent); }
    100% { box-shadow: 0 0 0 8px transparent; }
  }
</style>
