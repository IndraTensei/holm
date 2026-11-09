<script lang="ts">
  // A game's mark: its own logo (read from its exe) when Holm has it, otherwise a monogram tile in
  // the game's colour, so every game gets something that is clearly its own.
  import { hueOf, monogram } from '../gameLook';

  let { name, icon, hue, size = 56 }: { name: string; icon?: string; hue?: number; size?: number } = $props();

  const h = $derived(hue ?? hueOf(name));
  const letters = $derived(monogram(name));
</script>

{#if icon}
  <img class="mark logo" src={icon} alt="" style="--s:{size}px;--h:{h}" aria-hidden="true" />
{:else}
  <span class="mark tile display" style="--s:{size}px;--h:{h};--fs:{letters.length > 2 ? 0.34 : 0.42}" aria-hidden="true">{letters}</span>
{/if}

<style>
  .mark {
    flex: none;
    width: var(--s);
    height: var(--s);
    border-radius: calc(var(--s) * 0.25);
  }
  /* The real logo can arrive after the monogram: it fades in over it rather than snapping. */
  .logo {
    object-fit: contain;
    animation: appear 0.35s ease-out both;
    filter: drop-shadow(0 calc(var(--s) * 0.1) calc(var(--s) * 0.3) hsl(var(--h) 90% 55% / 0.35));
  }
  /* Like an app icon: a gradient in the game's colour, a soft top light, the initials in white. */
  .tile {
    display: grid;
    place-items: center;
    background:
      linear-gradient(160deg, rgb(255 255 255 / 0.22), transparent 45%),
      linear-gradient(135deg, hsl(var(--h) 88% 60%), hsl(calc(var(--h) + 32) 90% 42%));
    box-shadow:
      inset 0 0 0 1px rgb(255 255 255 / 0.14),
      0 calc(var(--s) * 0.1) calc(var(--s) * 0.3) hsl(var(--h) 90% 50% / 0.35);
    color: #fff;
    font-size: calc(var(--s) * var(--fs));
    font-weight: 800;
    letter-spacing: -0.04em;
    line-height: 1;
    text-shadow: 0 1px 2px rgb(0 0 0 / 0.25);
  }
  @keyframes appear {
    from {
      opacity: 0;
      transform: scale(0.85);
    }
  }
</style>
