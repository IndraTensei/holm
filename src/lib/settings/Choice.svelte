<script lang="ts" generics="T extends string | number">
  // A segmented control for two to four choices.
  let { value, options, onchange, label }: { value: T; options: { value: T; label: string }[]; onchange: (v: T) => void; label: string } = $props();
</script>

<div class="choice" role="radiogroup" aria-label={label}>
  {#each options as o (o.value)}
    <button role="radio" aria-checked={value === o.value} class:on={value === o.value} onclick={() => onchange(o.value)}>{o.label}</button>
  {/each}
</div>

<style>
  .choice {
    display: inline-flex;
    padding: 3px;
    border-radius: 10px;
    background: var(--fill-1);
  }
  button {
    height: 28px;
    padding: 0 12px;
    border-radius: 8px;
    font-size: var(--t-sm);
    font-weight: 550;
    color: var(--ink-2);
    white-space: nowrap;
  }
  button.on {
    background: var(--fill-3);
    color: var(--ink);
  }
</style>
