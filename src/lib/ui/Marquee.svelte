<script lang="ts">
  // Text that slides when it does not fit, and sits still when it does.
  // `once`: one quick pass to the end that then stays there, for text that changes every few
  // seconds (a lyric line), where a slow slide out and back would never finish.
  let { text, once = false, class: klass = '' }: { text: string; once?: boolean; class?: string } = $props();
  let box: HTMLDivElement;
  let inner = $state<HTMLSpanElement>();
  let overflow = $state(0);
  let slid = $state(false); // a `once` pass has moved the start out of view

  $effect(() => {
    text;
    slid = false;
    const span = inner;
    if (!span) return;
    // Fractional widths: scrollWidth rounds, which let a line 1-4 px too long sit clipped and still.
    const measure = () => {
      const o = span.getBoundingClientRect().width - box.getBoundingClientRect().width;
      overflow = o > 0.5 ? Math.ceil(o) : 0;
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(box);
    return () => ro.disconnect();
  });

  const style = $derived(
    once
      ? `--shift:-${overflow + 16}px;--time:${Math.min(3, Math.max(0.8, overflow / 45))}s`
      : `--shift:-${overflow + 16}px;--time:${Math.max(6, overflow / 18)}s`,
  );
</script>

<!-- Slides out and back once per text, then rests: an endless loop would keep the
     transparent island window painting at 60 fps (see stores/beat). The edge fades only
     while something is hidden behind it, so a line that fits shows its last letter whole. -->
<div class="marquee {klass}" class:cut={overflow > 0} class:slid bind:this={box} title={text}>
  {#key text}<span
      bind:this={inner}
      class:run={overflow > 0 && !once}
      class:once={overflow > 0 && once}
      {style}
      onanimationstart={() => (slid = once)}>{text}</span
    >{/key}
</div>

<style>
  .marquee {
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
  }
  .cut {
    mask-image: linear-gradient(90deg, #000 calc(100% - 14px), transparent);
  }
  .cut.slid {
    mask-image: linear-gradient(90deg, transparent, #000 14px, #000 calc(100% - 14px), transparent);
  }
  span {
    display: inline-block;
  }
  .run {
    animation: slide var(--time) ease-in-out 1.5s 2 alternate;
  }
  .once {
    animation: to-end var(--time) ease-in-out 0.6s forwards;
  }
  @keyframes slide {
    0%, 12% { transform: translateX(0); }
    88%, 100% { transform: translateX(var(--shift)); }
  }
  @keyframes to-end {
    to { transform: translateX(var(--shift)); }
  }
</style>
