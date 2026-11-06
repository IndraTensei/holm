<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, hint = '', children }: { label: string; hint?: string; children: Snippet } = $props();
</script>

<div class="field">
  <div class="text">
    <div class="label">{label}</div>
    {#if hint}<div class="hint">{hint}</div>{/if}
  </div>
  <div class="control">{@render children()}</div>
</div>

<style>
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 13px 0;
    border-bottom: 1px solid var(--line);
  }
  .field:last-child {
    border-bottom: 0;
  }
  .text {
    min-width: 0;
  }
  .label {
    font-size: var(--t-md);
    font-weight: 550;
  }
  .hint {
    margin-top: 2px;
    font-size: var(--t-sm);
    line-height: 1.45;
    color: var(--ink-3);
    max-width: 62ch;
  }
  .control {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
