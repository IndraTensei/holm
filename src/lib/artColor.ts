// The most vivid colour of an album cover, from a 24×24 sample. Done once per track.

import type { RGB } from './themes';

const cache = new Map<string, RGB | null>();

export function artColor(src: string): Promise<RGB | null> {
  if (cache.has(src)) return Promise.resolve(cache.get(src) ?? null);
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.decoding = 'async';
    img.onload = () => {
      try {
        const size = 24;
        const canvas = document.createElement('canvas');
        canvas.width = canvas.height = size;
        const g = canvas.getContext('2d', { willReadFrequently: true });
        if (!g) return resolve(null);
        g.drawImage(img, 0, 0, size, size);
        const data = g.getImageData(0, 0, size, size).data;
        // Hue histogram weighted by saturation and brightness: picks the cover's character
        // rather than its average mud.
        const bins = Array.from({ length: 18 }, () => ({ w: 0, r: 0, g: 0, b: 0 }));
        let grey = { w: 0, r: 0, g: 0, b: 0 };
        for (let i = 0; i < data.length; i += 4) {
          const r = data[i], gg = data[i + 1], b = data[i + 2];
          const max = Math.max(r, gg, b), min = Math.min(r, gg, b);
          const v = max / 255;
          const s = max === 0 ? 0 : (max - min) / max;
          if (s < 0.18 || v < 0.15) {
            grey = { w: grey.w + 1, r: grey.r + r, g: grey.g + gg, b: grey.b + b };
            continue;
          }
          let h = 0;
          const d = max - min;
          if (max === r) h = ((gg - b) / d + 6) % 6;
          else if (max === gg) h = (b - r) / d + 2;
          else h = (r - gg) / d + 4;
          const bin = bins[Math.floor((h / 6) * 18) % 18];
          const w = s * s * v;
          bin.w += w;
          bin.r += r * w;
          bin.g += gg * w;
          bin.b += b * w;
        }
        const best = bins.reduce((a, b) => (b.w > a.w ? b : a));
        const result: RGB | null =
          best.w > 2
            ? [best.r / best.w, best.g / best.w, best.b / best.w]
            : grey.w
              ? [grey.r / grey.w, grey.g / grey.w, grey.b / grey.w]
              : null;
        cache.set(src, result);
        if (cache.size > 60) cache.delete(cache.keys().next().value as string);
        resolve(result);
      } catch {
        // A remote cover without CORS taints the canvas.
        cache.set(src, null);
        resolve(null);
      }
    };
    img.onerror = () => resolve(null);
    img.src = src;
  });
}
