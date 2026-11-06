<script lang="ts">
  // Text that slides when it does not fit, and sits still when it does.
  let { text, class: klass = '' }: { text: string; class?: string } = $props();
  let box: HTMLDivElement;
  let inner = $state<HTMLSpanElement>();
  let overflow = $state(0);

  $effect(() => {
    text;
    const span = inner;
    if (!span) return;
    const measure = () => (overflow = Math.max(0, span.scrollWidth - box.clientWidth));
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(box);
    return () => ro.disconnect();
  });
</script>

<!-- Slides out and back once per text, then rests: an endless loop would keep the
     transparent island window painting at 60 fps (see stores/beat). -->
<div class="marquee {klass}" bind:this={box} title={text}>
  {#key text}<span bind:this={inner} class:run={overflow > 4} style="--shift:-{overflow + 16}px;--time:{Math.max(6, overflow / 18)}s">{text}</span>{/key}
</div>

<style>
  .marquee {
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
    mask-image: linear-gradient(90deg, #000 calc(100% - 14px), transparent);
  }
  span {
    display: inline-block;
  }
  .run {
    animation: slide var(--time) ease-in-out 1.5s 2 alternate;
  }
  @keyframes slide {
    0%, 12% { transform: translateX(0); }
    88%, 100% { transform: translateX(var(--shift)); }
  }
</style>
