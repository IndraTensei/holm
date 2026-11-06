// Spring motion for the island's shape, expressed as CSS `linear()` easing so the browser's
// compositor runs it: no JavaScript animation loop.

export interface Motion {
  easing: string;
  duration: number;
}

/** Samples a damped spring (mass 1) from 0 to 1 into a CSS linear() easing. */
export function spring(stiffness: number, damping: number): Motion {
  const dt = 1 / 240;
  let x = 0;
  let v = 0;
  let t = 0;
  const points: number[] = [];
  // Simulate until settled (or 2 s at most).
  while (t < 2) {
    const a = -stiffness * (x - 1) - damping * v;
    v += a * dt;
    x += v * dt;
    t += dt;
    points.push(x);
    if (Math.abs(x - 1) < 0.0015 && Math.abs(v) < 0.01 && t > 0.1) break;
  }
  const samples = 40;
  const step = Math.max(1, Math.floor(points.length / samples));
  const stops: string[] = ['0'];
  for (let i = step; i < points.length - 1; i += step) stops.push(points[i].toFixed(4));
  stops.push('1');
  return { easing: `linear(${stops.join(', ')})`, duration: Math.round(t * 1000) };
}

const cache = new Map<string, Motion>();

export function motionFor(kind: 'spring' | 'smooth' | 'reduced'): Motion {
  const hit = cache.get(kind);
  if (hit) return hit;
  const prefersReduced = typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;
  let motion: Motion;
  if (kind === 'reduced' || prefersReduced) motion = { easing: 'ease-out', duration: 140 };
  else if (kind === 'smooth') motion = { easing: 'cubic-bezier(0.32, 0.72, 0, 1)', duration: 460 };
  else motion = spring(230, 21);
  cache.set(kind, motion);
  return motion;
}
