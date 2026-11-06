<script lang="ts">
  // Seek bar: click or drag to move, a thicker track under the pointer.
  let { value, max, onseek, disabled = false }: { value: number; max: number; onseek: (v: number) => void; disabled?: boolean } = $props();
  let track: HTMLDivElement;
  let dragging = $state<number | null>(null);
  const shown = $derived(dragging ?? value);
  const pct = $derived(max > 0 ? Math.max(0, Math.min(1, shown / max)) * 100 : 0);

  function at(e: PointerEvent): number {
    const r = track.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * max;
  }
  function down(e: PointerEvent) {
    if (disabled || max <= 0) return;
    track.setPointerCapture(e.pointerId);
    dragging = at(e);
  }
  function move(e: PointerEvent) {
    if (dragging !== null) dragging = at(e);
  }
  function up() {
    if (dragging !== null) onseek(dragging);
    dragging = null;
  }
</script>

<div
  class="scrub"
  class:disabled
  class:dragging={dragging !== null}
  bind:this={track}
  role="slider"
  tabindex="0"
  aria-label="Position"
  aria-valuemin={0}
  aria-valuemax={Math.round(max)}
  aria-valuenow={Math.round(shown)}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onkeydown={(e) => {
    if (e.key === 'ArrowRight') onseek(Math.min(max, value + 5));
    if (e.key === 'ArrowLeft') onseek(Math.max(0, value - 5));
  }}
>
  <div class="rail"><div class="fill" style="transform:scaleX({pct / 100})"></div></div>
</div>

<style>
  .scrub {
    height: 16px;
    display: flex;
    align-items: center;
    cursor: pointer;
    touch-action: none;
  }
  .disabled {
    cursor: default;
  }
  .rail {
    position: relative;
    width: 100%;
    height: 4px;
    border-radius: 3px;
    background: var(--fill-2);
    overflow: hidden;
    transition: height 0.18s var(--ease-out);
  }
  .scrub:hover .rail,
  .dragging .rail {
    height: 7px;
  }
  /* Stepped with the 4 Hz clock, never transitioned: a transition restarted every tick ran at
     60 fps for as long as music played (a 0.4 px step is invisible). A transform needs no layout. */
  .fill {
    height: 100%;
    background: var(--ink);
    border-radius: 3px;
    transform-origin: left;
  }
  .dragging .fill {
    transition: none;
    background: var(--accent);
  }
</style>
