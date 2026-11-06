<script lang="ts">
  // "Opening Valorant": a one-off, four-second intro when a game comes to the front. The name
  // rises in letter by letter, a light sweeps across, a bar charges, and the island says which
  // keys bring it back or hide it. Then Rust steps the island aside for the game.
  // Plays once per launch, so plain CSS animations are fine here (no loops: see stores/beat).
  import { Gamepad2 } from '@lucide/svelte';
  import { settings } from '../stores/settings.svelte';

  let { name }: { name: string } = $props();

  // A hue per game: a few well-known ones tuned by hand, the rest from the name.
  const HUES: Record<string, number> = {
    valorant: 355,
    'grand theft auto v': 135,
    'red dead redemption 2': 8,
    'counter-strike 2': 38,
    fortnite: 270,
    'apex legends': 2,
    'league of legends': 45,
    minecraft: 110,
    'elden ring': 42,
    'cyberpunk 2077': 58,
  };
  function hueOf(n: string): number {
    const known = HUES[n.toLowerCase()];
    if (known !== undefined) return known;
    let h = 0;
    for (const c of n) h = (h * 31 + c.charCodeAt(0)) % 360;
    return h;
  }

  const hue = $derived(hueOf(name));
  const letters = $derived([...name]);
  const keys = (combo: string) => (combo ? combo.split('+') : []);
  const back = $derived(keys(settings.v.hotkey));
  const hide = $derived(keys(settings.v.hideHotkey));
</script>

<div class="intro" style="--hue:{hue}" role="status" aria-label="Opening {name}">
  <div class="sweep" aria-hidden="true"></div>
  <div class="top">
    <span class="icon"><Gamepad2 size={14} strokeWidth={2.4} /></span>
    <span>Opening</span>
  </div>
  <div class="name display" aria-hidden="true">
    {#each letters as letter, i}
      <span class="ch" style="animation-delay:{180 + i * 38}ms">{letter === ' ' ? ' ' : letter}</span>
    {/each}
    <span class="glint">{name}</span>
  </div>
  <div class="charge" aria-hidden="true"><span></span></div>
  <div class="keys">
    {#if back.length}
      <span class="combo">{#each back as k}<kbd>{k}</kbd>{/each}<em>brings me back</em></span>
    {/if}
    {#if hide.length}
      <span class="combo">{#each hide as k}<kbd>{k}</kbd>{/each}<em>hides me</em></span>
    {/if}
  </div>
</div>

<style>
  .intro {
    --g: hsl(var(--hue) 92% 62%);
    --g-soft: hsl(var(--hue) 92% 62% / 0.22);
    position: relative;
    width: 460px;
    padding: 16px 22px 16px;
    overflow: hidden;
    color: var(--ink);
    animation: out 0.45s ease-in 3.75s forwards;
  }
  /* A glow from below in the game's colour, and one light sweep across the island. */
  .intro::before {
    content: '';
    position: absolute;
    inset: 0;
    background: radial-gradient(120% 90% at 50% 120%, var(--g-soft), transparent 70%);
    opacity: 0;
    animation: fade 0.8s ease-out 0.1s forwards;
  }
  .sweep {
    position: absolute;
    top: -40%;
    bottom: -40%;
    width: 38%;
    left: -45%;
    background: linear-gradient(100deg, transparent, hsl(var(--hue) 100% 85% / 0.16), transparent);
    transform: skewX(-18deg);
    animation: sweep 1.1s cubic-bezier(0.3, 0.6, 0.2, 1) 0.55s forwards;
  }
  .top {
    position: relative;
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: var(--t-xs);
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--g);
    opacity: 0;
    animation: rise 0.4s ease-out 0.05s forwards;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: var(--g-soft);
  }
  .name {
    position: relative;
    margin-top: 8px;
    font-size: 30px;
    font-weight: 800;
    letter-spacing: -0.02em;
    line-height: 1.1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ch {
    display: inline-block;
    white-space: pre;
    opacity: 0;
    transform: translateY(55%) scale(0.92);
    filter: blur(6px);
    animation: letter 0.5s cubic-bezier(0.2, 0.9, 0.25, 1.15) forwards;
  }
  /* The same text again, clipped to a moving highlight: a glint once the name has landed. */
  .glint {
    position: absolute;
    inset: 0;
    color: transparent;
    background: linear-gradient(100deg, transparent 35%, rgb(255 255 255 / 0.85) 50%, transparent 65%) no-repeat;
    background-size: 250% 100%;
    background-position: 130% 0;
    -webkit-background-clip: text;
    background-clip: text;
    animation: glint 0.9s ease-in-out 1.35s forwards;
  }
  .charge {
    position: relative;
    margin-top: 12px;
    height: 3px;
    border-radius: 2px;
    background: var(--fill-2);
    overflow: hidden;
  }
  .charge span {
    position: absolute;
    inset: 0;
    border-radius: 2px;
    background: linear-gradient(90deg, hsl(var(--hue) 92% 50%), var(--g), hsl(calc(var(--hue) + 30) 95% 70%));
    box-shadow: 0 0 12px var(--g);
    transform: scaleX(0);
    transform-origin: left;
    animation: charge 2.9s cubic-bezier(0.5, 0, 0.2, 1) 0.4s forwards;
  }
  .keys {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin-top: 12px;
    font-size: var(--t-xs);
    color: var(--ink-2);
    opacity: 0;
    animation: rise 0.45s ease-out 1.25s forwards;
  }
  .combo {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  kbd {
    min-width: 20px;
    padding: 2px 6px;
    border-radius: 6px;
    background: var(--fill-2);
    box-shadow: inset 0 -1.5px 0 rgb(0 0 0 / 0.35);
    font: 600 11px/1.3 var(--font-ui);
    color: var(--ink);
    text-align: center;
  }
  em {
    margin-left: 5px;
    font-style: normal;
  }
  @keyframes letter {
    to {
      opacity: 1;
      transform: none;
      filter: none;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @keyframes fade {
    to {
      opacity: 1;
    }
  }
  @keyframes sweep {
    to {
      left: 120%;
    }
  }
  @keyframes glint {
    to {
      background-position: -40% 0;
    }
  }
  @keyframes charge {
    to {
      transform: scaleX(1);
    }
  }
  @keyframes out {
    to {
      opacity: 0;
      filter: blur(4px);
      transform: scale(0.98);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .intro *,
    .intro::before {
      animation-duration: 1ms !important;
      animation-delay: 0ms !important;
    }
  }
</style>
