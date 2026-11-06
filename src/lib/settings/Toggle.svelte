<script lang="ts">
  let { checked, onchange, label }: { checked: boolean; onchange: (v: boolean) => void; label: string } = $props();
</script>

<button class="toggle" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}>
  <span></span>
</button>

<style>
  .toggle {
    width: 40px;
    height: 24px;
    border-radius: 999px;
    background: var(--fill-3);
    padding: 3px;
    transition: background 0.2s;
  }
  .toggle span {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.25s var(--ease-out);
  }
  .on {
    background: var(--accent);
  }
  .on span {
    transform: translateX(16px);
  }
</style>
