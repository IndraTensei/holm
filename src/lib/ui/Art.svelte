<script lang="ts">
  import { Music2 } from '@lucide/svelte';

  let { src, size = 40, radius = 10, alt = '' }: { src: string | null; size?: number; radius?: number; alt?: string } = $props();
  let failed = $state(false);
  $effect(() => {
    src;
    failed = false;
  });
</script>

<div class="art" style="width:{size}px;height:{size}px;border-radius:{radius}px">
  {#if src && !failed}
    <img {src} {alt} draggable="false" onerror={() => (failed = true)} />
  {:else}
    <Music2 size={Math.round(size * 0.45)} strokeWidth={1.8} />
  {/if}
</div>

<style>
  .art {
    flex: none;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--fill-2);
    color: var(--ink-3);
    box-shadow: inset 0 0 0 0.5px var(--line);
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
