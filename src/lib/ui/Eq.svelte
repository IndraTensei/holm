<script lang="ts">
  // A small equalizer that dances while music plays and rests flat when it pauses.
  // It steps on the shared beat (8 fps) instead of a 60 fps CSS loop: see stores/beat.
  import { beat } from '../stores/beat.svelte';

  let { playing = true, bars = 4, height = 14, color = 'var(--accent)' }: { playing?: boolean; bars?: number; height?: number; color?: string } = $props();

  // 11 levels walked with a different stride per bar (all coprime with 11), so no two bars move together.
  const levels = [0.3, 0.85, 0.5, 0.95, 0.4, 0.7, 0.25, 0.9, 0.6, 0.35, 0.78];
  const strides = [1, 3, 2, 4, 5, 7];

  $effect(() => (playing ? beat.use() : undefined));

  function level(i: number, n: number): number {
    return playing ? levels[(n * strides[i % strides.length] + i * 4) % levels.length] : 0.22;
  }
</script>

<div class="eq" class:playing style="height:{height}px;--c:{color}" aria-hidden="true">
  {#each Array(bars) as _, i}
    <span style="transform:scaleY({level(i, beat.n)})"></span>
  {/each}
</div>

<style>
  /* Its own compositing layer: each beat repaints these pixels, not the whole island. */
  .eq {
    will-change: transform;
    display: inline-flex;
    align-items: center;
    gap: 2.5px;
  }
  span {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--c);
    transition: transform 0.4s var(--ease-out);
  }
  /* While playing, each step jumps: a transition here would bring back 60 fps frames. */
  .playing span {
    transition: none;
  }
</style>
