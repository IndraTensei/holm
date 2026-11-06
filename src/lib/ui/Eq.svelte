<script lang="ts">
  // A small equalizer that dances while music plays and rests flat when it pauses.
  let { playing = true, bars = 4, height = 14, color = 'var(--accent)' }: { playing?: boolean; bars?: number; height?: number; color?: string } = $props();
  const timings = [0.92, 0.71, 1.13, 0.83, 1.02, 0.77];
</script>

<div class="eq" class:playing style="height:{height}px;--c:{color}" aria-hidden="true">
  {#each Array(bars) as _, i}
    <span style="animation-duration:{timings[i % timings.length]}s;animation-delay:{-i * 0.17}s"></span>
  {/each}
</div>

<style>
  .eq {
    display: inline-flex;
    align-items: center;
    gap: 2.5px;
  }
  span {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--c);
    transform: scaleY(0.22);
    transition: transform 0.4s var(--ease-out);
  }
  .playing span {
    animation: bounce 1s ease-in-out infinite alternate;
  }
  @keyframes bounce {
    0% { transform: scaleY(0.25); }
    35% { transform: scaleY(0.95); }
    60% { transform: scaleY(0.45); }
    100% { transform: scaleY(0.8); }
  }
</style>
