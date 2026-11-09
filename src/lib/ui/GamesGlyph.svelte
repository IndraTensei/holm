<script lang="ts">
  // Holm's own mark for Games in the settings sidebar: an aiming reticle (a ring broken by four
  // sight lines, a dot in the middle), drawn to sit with the other sidebar icons (24 grid, 2 px
  // round strokes) without being the stock gamepad.
  let { size = 16 }: { size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#each [0, 90, 180, 270] as turn}
    <g transform="rotate({turn} 12 12)">
      <path d="M18.58 9.61A7 7 0 0 0 14.39 5.42" />
      <path d="M12 2.5v3.5" />
    </g>
  {/each}
  <circle cx="12" cy="12" r="1.4" fill="currentColor" stroke="none" />
</svg>
