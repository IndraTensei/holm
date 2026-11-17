<script lang="ts">
  let { data, width = 120, height = 32, max = 0, color = 'var(--accent)' }: { data: number[]; width?: number; height?: number; max?: number; color?: string } = $props();
  const id = `spark-${Math.random().toString(36).slice(2, 8)}`;
  const path = $derived.by(() => {
    if (data.length < 2) return { line: '', area: '' };
    const top = max || Math.max(...data, 1);
    const step = width / (data.length - 1);
    const pts = data.map((v, i) => [i * step, height - 1.5 - (Math.min(v, top) / top) * (height - 3)]);
    const line = pts.map((p, i) => `${i ? 'L' : 'M'}${p[0].toFixed(1)} ${p[1].toFixed(1)}`).join(' ');
    return { line, area: `${line} L${width} ${height} L0 ${height} Z` };
  });
</script>

<svg {width} {height} viewBox="0 0 {width} {height}" aria-hidden="true" overflow="visible">
  <defs>
    <!-- The theme's glow on the line (--glow-a, 0 = none), with room so the blur is never clipped. -->
    <filter id="{id}-glow" filterUnits="userSpaceOnUse" x={-8} y={-8} width={width + 16} height={height + 16}>
      <feDropShadow dx="0" dy="0" stdDeviation="1.6" style="flood-color:{color};flood-opacity:var(--glow-a, 0)" />
    </filter>
    <linearGradient {id} x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color={color} stop-opacity="0.28" />
      <stop offset="1" stop-color={color} stop-opacity="0" />
    </linearGradient>
  </defs>
  <path d={path.area} fill="url(#{id})" />
  <path d={path.line} fill="none" stroke={color} stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round" filter="url(#{id}-glow)" />
</svg>
