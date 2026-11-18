// Themes: a palette, a type pairing and a look. The island itself stays a solid object (like the
// hardware it imitates); themes change its fill, ink, accent and the rim light around it, and the look
// gives each one its own light, texture, glow and motion (see LOOKS).

import type { Feel } from './motion';

export interface Theme {
  id: string;
  name: string;
  mode: 'dark' | 'light';
  island: string;
  ink: string;
  accent: string;
  ok: string;
  warn: string;
  err: string;
  /** Strength of the thin light around the island edge, 0–1. */
  rim: number;
  fontUi: string;
  fontDisplay: string;
  /** Take accent and tint from the album art when music plays. */
  adaptive?: boolean;
  custom?: boolean;
  /** How the theme is drawn, beyond colour. Everything here is static paint or plays once. */
  look?: Look;
}

export interface Look {
  /** Images layered over the island fill: light, grain, scanlines. In notch style they fade in below the top edge. */
  surface?: string;
  /** Outer box-shadow layers on the island: a glow, a coloured shadow. */
  edge?: string;
  /** Inset box-shadow layers inside the island's edge: a highlight, a shade. */
  frame?: string;
  /** A 1 px line around the island (pill style; in notch style the island flows into the screen edge). */
  outline?: string;
  /** Scale for card and art corners, 1 or more (1 = today). */
  round?: number;
  /** Strength (0–1) of the accent glow on rings and sparklines. */
  glow?: number;
  /** A text-shadow for the big numbers (the Home clock, System figures, the Focus timer). */
  textGlow?: string;
  /** Display text in italics (fonts with a real italic: Fraunces, Instrument Serif). */
  italic?: boolean;
  /** Display text only in weights the font has (Instrument Serif has one), never a faux bold. */
  trueWeights?: boolean;
  tracking?: string;
  /** Progress and level bars. */
  bar?: { height?: string; fill?: string };
  /** The main round button (play/pause). */
  solid?: { bg: string; ink: string; ring?: string };
  /** How the island moves when it opens and closes (with the Spring motion setting). */
  feel?: Feel;
}

/** The accent at an opacity (the album colour, for adaptive themes). */
const A = (a: number) => `rgb(from var(--accent) r g b / ${a})`;
const C = (hex: string, a: number) => `rgb(from ${hex} r g b / ${a})`;
const grain = (light: boolean, a: number) =>
  `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='160'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2' stitchTiles='stitch'/%3E%3CfeColorMatrix values='0 0 0 0 ${light ? 0 : 1} 0 0 0 0 ${light ? 0 : 1} 0 0 0 0 ${light ? 0 : 1} 0 0 0 ${a} 0'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E")`;

// Each theme's look. All of it is static paint, or motion that plays once when the island opens or
// closes: nothing loops, so a resting island costs what it did before.
const LOOKS: Record<string, Look> = {
  // The default stays close to the hardware: a glassy sheen, and a bar in the album's colour.
  obsidian: {
    surface: 'radial-gradient(120% 70% at 50% 0%, rgb(255 255 255 / 0.07), transparent 60%)',
    frame: 'inset 0 1px 0 rgb(255 255 255 / 0.05)',
    edge: `0 10px 30px -14px ${A(0.45)}`,
    glow: 0.55,
    bar: { fill: `linear-gradient(90deg, ${A(0.65)}, var(--accent))` },
  },
  // Brushed metal.
  graphite: {
    surface:
      'linear-gradient(180deg, rgb(255 255 255 / 0.06), transparent 45%), repeating-linear-gradient(90deg, rgb(255 255 255 / 0.018) 0 1px, transparent 1px 3px)',
    frame: 'inset 0 1px 0 rgb(255 255 255 / 0.1), inset 0 -1px 0 rgb(0 0 0 / 0.4)',
    bar: { fill: 'linear-gradient(90deg, #6fb6e8, var(--accent))' },
    solid: { bg: 'linear-gradient(180deg, #fafafc, #c3c6cf)', ink: '#1d1d20', ring: 'inset 0 1px 0 #fff, 0 2px 6px rgb(0 0 0 / 0.5)' },
    glow: 0.5,
    feel: 'crisp',
  },
  // Glazed ceramic.
  porcelain: {
    surface: 'radial-gradient(130% 90% at 30% 0%, #ffffff, transparent 60%), radial-gradient(90% 90% at 100% 100%, rgb(53 87 245 / 0.06), transparent 60%)',
    frame: 'inset 0 1px 0 #ffffff, inset 0 -1px 0 rgb(0 0 0 / 0.05)',
    edge: `0 14px 34px -14px ${A(0.4)}`,
    bar: { fill: 'linear-gradient(90deg, #6d8bff, var(--accent))' },
    solid: { bg: 'linear-gradient(160deg, #5a78ff, #2a45d8)', ink: '#ffffff', ring: `0 6px 14px -6px ${A(0.7)}` },
  },
  // A gold spotlight from above.
  arena: {
    surface: `radial-gradient(80% 110% at 50% -25%, ${A(0.2)}, transparent 60%), linear-gradient(0deg, rgb(70 227 183 / 0.06), transparent 40%)`,
    outline: A(0.32),
    edge: `0 0 26px -8px ${A(0.55)}`,
    glow: 0.7,
    textGlow: `0 0 18px ${A(0.4)}`,
    tracking: '0.01em',
    bar: { fill: 'linear-gradient(90deg, #c8912a, #ffe08a)' },
    solid: { bg: 'linear-gradient(135deg, #ffe7a3, #f4c454 45%, #c48c22)', ink: '#0a1024', ring: `0 0 18px -4px ${A(0.8)}` },
    feel: 'crisp',
  },
  sakura: {
    surface: `radial-gradient(120% 130% at 0% 0%, ${A(0.13)}, transparent 55%), radial-gradient(80% 100% at 100% 100%, rgb(255 196 214 / 0.45), transparent 65%)`,
    frame: 'inset 0 -1px 0 rgb(255 255 255 / 0.9)',
    edge: '0 12px 32px -12px rgb(217 79 124 / 0.5)',
    round: 1.7,
    italic: true,
    bar: { height: '6px', fill: 'var(--accent)' },
    solid: { bg: 'var(--accent)', ink: '#fff', ring: `0 6px 16px -6px ${A(0.8)}` },
    feel: 'bouncy',
  },
  // Candlelight on paper.
  editorial: {
    surface: `radial-gradient(110% 140% at 0% 0%, ${A(0.16)}, transparent 55%), ${grain(false, 0.07)}`,
    outline: A(0.22),
    edge: `0 0 30px -10px ${A(0.5)}`,
    glow: 0.6,
    textGlow: `0 0 20px ${A(0.3)}`,
    italic: true,
    trueWeights: true,
    bar: { fill: 'linear-gradient(90deg, #b8894a, #f6dca4)' },
    solid: { bg: 'linear-gradient(135deg, #f8e2b0, #e9c27e 50%, #b8894a)', ink: '#000', ring: `0 0 16px -4px ${A(0.6)}` },
    feel: 'slow',
  },
  // An aurora.
  nord: {
    surface: `radial-gradient(110% 90% at 100% 0%, ${C('#88c0d0', 0.18)}, transparent 55%), radial-gradient(90% 90% at 0% 100%, ${C('#a3be8c', 0.12)}, transparent 60%), radial-gradient(60% 70% at 15% 0%, ${C('#b48ead', 0.1)}, transparent 60%)`,
    frame: 'inset 0 1px 0 rgb(236 239 244 / 0.07)',
    edge: `0 0 28px -10px ${C('#88c0d0', 0.5)}`,
    glow: 0.6,
    bar: { fill: 'linear-gradient(90deg, #a3be8c, #88c0d0, #b48ead)' },
    solid: { bg: 'linear-gradient(135deg, #8fbcbb, #88c0d0 50%, #81a1c1)', ink: '#2e3440' },
    feel: 'slow',
  },
  dracula: {
    surface: `radial-gradient(100% 120% at 100% 0%, ${C('#ff79c6', 0.13)}, transparent 55%), radial-gradient(120% 120% at 0% 100%, ${A(0.14)}, transparent 60%)`,
    outline: A(0.3),
    edge: `0 0 28px -8px ${A(0.55)}`,
    glow: 0.8,
    textGlow: `0 0 16px ${A(0.4)}`,
    bar: { fill: 'linear-gradient(90deg, var(--accent), #ff79c6)' },
    solid: { bg: 'linear-gradient(135deg, #ff79c6, #bd93f9)', ink: '#282a36', ring: `0 0 16px -3px ${C('#ff79c6', 0.7)}` },
    feel: 'bouncy',
  },
  // Pastel and cosy.
  catppuccin: {
    surface: `radial-gradient(100% 120% at 0% 0%, ${A(0.13)}, transparent 55%), radial-gradient(100% 120% at 100% 100%, ${C('#89dceb', 0.1)}, transparent 60%)`,
    frame: 'inset 0 1px 0 rgb(205 214 244 / 0.06)',
    edge: `0 12px 30px -12px ${A(0.45)}`,
    round: 1.35,
    glow: 0.5,
    bar: { height: '5px', fill: 'linear-gradient(90deg, #89dceb, #cba6f7, #f5c2e7)' },
    solid: { bg: 'linear-gradient(135deg, #f5c2e7, #cba6f7)', ink: '#1e1e2e', ring: `0 6px 16px -6px ${A(0.7)}` },
    feel: 'bouncy',
  },
  // City lights at night.
  'tokyo-night': {
    surface: `linear-gradient(180deg, transparent 45%, ${A(0.1)}), radial-gradient(60% 90% at 85% 110%, ${C('#bb9af7', 0.18)}, transparent 70%)`,
    outline: A(0.26),
    edge: `0 0 28px -8px ${A(0.55)}`,
    glow: 0.75,
    textGlow: `0 0 18px ${A(0.35)}`,
    bar: { fill: 'linear-gradient(90deg, #7dcfff, #7aa2f7, #bb9af7)' },
    solid: { bg: 'linear-gradient(135deg, #7dcfff, #7aa2f7 50%, #bb9af7)', ink: '#1a1b26', ring: `0 0 16px -4px ${A(0.75)}` },
  },
  // Rose and iris.
  'rose-pine': {
    surface: `radial-gradient(110% 130% at 100% 0%, ${A(0.13)}, transparent 55%), radial-gradient(100% 120% at 0% 100%, ${C('#c4a7e7', 0.11)}, transparent 60%)`,
    frame: 'inset 0 1px 0 rgb(224 222 244 / 0.06)',
    edge: `0 12px 30px -12px ${A(0.4)}`,
    italic: true,
    glow: 0.5,
    bar: { fill: 'linear-gradient(90deg, #c4a7e7, #ebbcba)' },
    solid: { bg: 'linear-gradient(135deg, #f6c177, #ebbcba 50%, #c4a7e7)', ink: '#191724' },
    feel: 'slow',
  },
  // Warm amber light: from above, and an ember glow rising from the bottom edge.
  gruvbox: {
    surface: `radial-gradient(120% 140% at 50% 0%, ${A(0.09)}, transparent 60%), radial-gradient(90% 70% at 50% 120%, ${C('#fe8019', 0.16)}, transparent 70%)`,
    outline: A(0.16),
    edge: `0 0 26px -10px ${C('#fe8019', 0.5)}`,
    glow: 0.5,
    bar: { fill: 'linear-gradient(90deg, #b8bb26, #fabd2f, #fe8019)' },
    solid: { bg: 'linear-gradient(135deg, #fabd2f, #fe8019)', ink: '#282828' },
    feel: 'crisp',
  },
  // Tea leaves.
  matcha: {
    surface: `radial-gradient(120% 120% at 0% 0%, ${A(0.13)}, transparent 55%), ${grain(false, 0.05)}`,
    frame: 'inset 0 1px 0 rgb(228 238 225 / 0.06)',
    edge: `0 12px 30px -12px ${A(0.35)}`,
    round: 1.25,
    bar: { height: '5px', fill: 'linear-gradient(90deg, #7fd6a4, #a9d38c)' },
    solid: { bg: 'linear-gradient(135deg, #d3ebb4, #a9d38c 50%, #7fb46a)', ink: '#151d17' },
    feel: 'slow',
  },
  synthwave: {
    surface: `linear-gradient(180deg, transparent 35%, ${A(0.13)} 100%), repeating-linear-gradient(0deg, rgb(255 255 255 / 0.03) 0 1px, transparent 1px 3px)`,
    outline: A(0.5),
    edge: `0 0 30px -6px ${A(0.75)}`,
    glow: 0.85,
    textGlow: `0 0 16px ${A(0.55)}`,
    tracking: '0.01em',
    bar: { fill: 'linear-gradient(90deg, #36f2c6, var(--accent))' },
    solid: { bg: 'linear-gradient(135deg, var(--accent), #8a5bff)', ink: '#fff', ring: `0 0 18px -2px ${A(0.8)}` },
    feel: 'bouncy',
  },
  // Warm sunlight on paper.
  'solarized-light': {
    surface: `radial-gradient(120% 140% at 100% 0%, ${C('#b58900', 0.12)}, transparent 55%), ${grain(true, 0.04)}`,
    frame: 'inset 0 1px 0 rgb(255 255 255 / 0.85)',
    edge: `0 12px 30px -14px ${A(0.4)}`,
    bar: { fill: 'linear-gradient(90deg, #2aa198, #268bd2, #6c71c4)' },
    solid: { bg: 'linear-gradient(135deg, #2aa198, #268bd2)', ink: '#ffffff', ring: `0 6px 14px -6px ${A(0.6)}` },
  },
};

const base = { ok: '#5ce39b', warn: '#ffc35a', err: '#ff6b6b', rim: 0.35 };

const PALETTES: Omit<Theme, 'look'>[] = [
  { ...base, id: 'obsidian', name: 'Obsidian', mode: 'dark', island: '#000000', ink: '#f5f5f7', accent: '#9fb3ff', fontUi: 'geist', fontDisplay: 'bricolage', adaptive: true },
  { ...base, id: 'graphite', name: 'Graphite', mode: 'dark', island: '#1d1d20', ink: '#ececf1', accent: '#8fd3ff', fontUi: 'inter', fontDisplay: 'inter', rim: 0.2 },
  { ...base, id: 'porcelain', name: 'Porcelain', mode: 'light', island: '#f6f6f3', ink: '#17171c', accent: '#3557f5', ok: '#118a4e', warn: '#b86e00', err: '#d33a3a', fontUi: 'geist', fontDisplay: 'bricolage', rim: 0.5 },
  { ...base, id: 'arena', name: 'Arena', mode: 'dark', island: '#0a1024', ink: '#eef2ff', accent: '#f4c454', ok: '#46e3b7', fontUi: 'sora', fontDisplay: 'space-grotesk', rim: 0.55 },
  { ...base, id: 'sakura', name: 'Sakura', mode: 'light', island: '#fff4f6', ink: '#3b2230', accent: '#d94f7c', ok: '#2f9e6a', warn: '#c07a10', err: '#cf3b4b', fontUi: 'dm-sans', fontDisplay: 'fraunces', rim: 0.45 },
  { ...base, id: 'editorial', name: 'Editorial', mode: 'dark', island: '#000000', ink: '#efeae0', accent: '#e9c27e', fontUi: 'manrope', fontDisplay: 'instrument', rim: 0.25 },
  { ...base, id: 'nord', name: 'Nord', mode: 'dark', island: '#2e3440', ink: '#eceff4', accent: '#88c0d0', ok: '#a3be8c', warn: '#ebcb8b', err: '#bf616a', fontUi: 'inter', fontDisplay: 'outfit', rim: 0.3 },
  { ...base, id: 'dracula', name: 'Dracula', mode: 'dark', island: '#282a36', ink: '#f8f8f2', accent: '#bd93f9', ok: '#50fa7b', warn: '#f1fa8c', err: '#ff5555', fontUi: 'jakarta', fontDisplay: 'jakarta', rim: 0.35 },
  { ...base, id: 'catppuccin', name: 'Catppuccin Mocha', mode: 'dark', island: '#1e1e2e', ink: '#cdd6f4', accent: '#cba6f7', ok: '#a6e3a1', warn: '#f9e2af', err: '#f38ba8', fontUi: 'outfit', fontDisplay: 'outfit', rim: 0.35 },
  { ...base, id: 'tokyo-night', name: 'Tokyo Night', mode: 'dark', island: '#1a1b26', ink: '#c0caf5', accent: '#7aa2f7', ok: '#9ece6a', warn: '#e0af68', err: '#f7768e', fontUi: 'geist', fontDisplay: 'space-grotesk', rim: 0.4 },
  { ...base, id: 'rose-pine', name: 'Rosé Pine', mode: 'dark', island: '#191724', ink: '#e0def4', accent: '#ebbcba', ok: '#9ccfd8', warn: '#f6c177', err: '#eb6f92', fontUi: 'manrope', fontDisplay: 'fraunces', rim: 0.35 },
  { ...base, id: 'gruvbox', name: 'Gruvbox', mode: 'dark', island: '#282828', ink: '#ebdbb2', accent: '#fabd2f', ok: '#b8bb26', warn: '#fe8019', err: '#fb4934', fontUi: 'jetbrains', fontDisplay: 'jetbrains', rim: 0.25 },
  { ...base, id: 'matcha', name: 'Matcha', mode: 'dark', island: '#151d17', ink: '#e4eee1', accent: '#a9d38c', ok: '#7fd6a4', fontUi: 'dm-sans', fontDisplay: 'bricolage', rim: 0.3 },
  { ...base, id: 'synthwave', name: 'Synthwave', mode: 'dark', island: '#150c26', ink: '#f6ecff', accent: '#ff5bd6', ok: '#36f2c6', warn: '#ffe66d', err: '#ff5a76', fontUi: 'space-grotesk', fontDisplay: 'space-grotesk', rim: 0.7 },
  { ...base, id: 'solarized-light', name: 'Solarized Light', mode: 'light', island: '#fdf6e3', ink: '#3d4f57', accent: '#268bd2', ok: '#5f8700', warn: '#b58900', err: '#dc322f', fontUi: 'inter', fontDisplay: 'inter', rim: 0.5 },
];

/** Built-in themes. A custom theme copies its starting theme, look included. */
export const THEMES: Theme[] = PALETTES.map((t) => ({ ...t, look: LOOKS[t.id] }));

export interface FontOption {
  id: string;
  label: string;
  stack: string;
}

const fallback = "'Segoe UI Variable Text', 'Segoe UI', system-ui, sans-serif";

export const FONTS: FontOption[] = [
  { id: 'geist', label: 'Geist', stack: `'Geist Variable', ${fallback}` },
  { id: 'inter', label: 'Inter', stack: `'Inter Variable', ${fallback}` },
  { id: 'bricolage', label: 'Bricolage Grotesque', stack: `'Bricolage Grotesque Variable', ${fallback}` },
  { id: 'manrope', label: 'Manrope', stack: `'Manrope Variable', ${fallback}` },
  { id: 'jakarta', label: 'Plus Jakarta Sans', stack: `'Plus Jakarta Sans Variable', ${fallback}` },
  { id: 'space-grotesk', label: 'Space Grotesk', stack: `'Space Grotesk Variable', ${fallback}` },
  { id: 'outfit', label: 'Outfit', stack: `'Outfit Variable', ${fallback}` },
  { id: 'sora', label: 'Sora', stack: `'Sora Variable', ${fallback}` },
  { id: 'dm-sans', label: 'DM Sans', stack: `'DM Sans Variable', ${fallback}` },
  { id: 'fraunces', label: 'Fraunces', stack: `'Fraunces Variable', Georgia, serif` },
  { id: 'instrument', label: 'Instrument Serif', stack: `'Instrument Serif', Georgia, serif` },
  { id: 'jetbrains', label: 'JetBrains Mono', stack: `'JetBrains Mono Variable', Consolas, monospace` },
  { id: 'geist-mono', label: 'Geist Mono', stack: `'Geist Mono Variable', Consolas, monospace` },
  { id: 'system', label: 'Segoe UI (system)', stack: fallback },
];

export function fontStack(id: string): string {
  return (FONTS.find((f) => f.id === id) ?? FONTS[0]).stack;
}

export function findTheme(id: string, custom: Theme[] = []): Theme {
  return custom.find((t) => t.id === id) ?? THEMES.find((t) => t.id === id) ?? THEMES[0];
}

// ---------------------------------------------------------------- color math

export type RGB = [number, number, number];

export function hexToRgb(hex: string): RGB {
  const h = hex.replace('#', '');
  const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h.padEnd(6, '0');
  return [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16)) as RGB;
}

export function rgbToHex([r, g, b]: RGB): string {
  return '#' + [r, g, b].map((v) => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, '0')).join('');
}

export function rgbToHsl([r, g, b]: RGB): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h * 60, s, l];
}

export function hslToRgb(h: number, s: number, l: number): RGB {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return [f(0) * 255, f(8) * 255, f(4) * 255];
}

export function mix(a: string, b: string, t: number): string {
  const x = hexToRgb(a), y = hexToRgb(b);
  return rgbToHex([0, 1, 2].map((i) => x[i] + (y[i] - x[i]) * t) as RGB);
}

export function alpha(hex: string, a: number): string {
  const [r, g, b] = hexToRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

function luminance(hex: string): number {
  const c = hexToRgb(hex).map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}

export function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)].sort((m, n) => n - m);
  return (x + 0.05) / (y + 0.05);
}

/** Readable text color to put on top of an accent fill. */
export function inkOn(fill: string): string {
  return contrast(fill, '#000000') >= contrast(fill, '#ffffff') ? '#0b0b0f' : '#ffffff';
}

/** Album colour → an accent that reads on the island, keeping the art's hue. */
export function accentFromArt(rgb: RGB, mode: 'dark' | 'light'): { accent: string; tint: string } {
  const [h, s] = rgbToHsl(rgb);
  const sat = Math.max(0.45, Math.min(0.9, s * 1.15));
  const accent = rgbToHex(hslToRgb(h, sat, mode === 'dark' ? 0.7 : 0.42));
  const tint = rgbToHex(hslToRgb(h, Math.min(0.5, sat * 0.6), mode === 'dark' ? 0.07 : 0.95));
  return { accent, tint };
}

export interface ThemeVars {
  [key: string]: string;
}

/** CSS custom properties for a theme, with every derived shade computed here once. */
export function themeVars(theme: Theme, opts: { fontUi?: string; fontDisplay?: string; accent?: string; tint?: string }): ThemeVars {
  const island = opts.tint ?? theme.island;
  const accent = opts.accent ?? theme.accent;
  const dark = theme.mode === 'dark';
  const ink = theme.ink;
  return {
    '--island': island,
    '--ink': ink,
    '--ink-2': alpha(ink, dark ? 0.66 : 0.7),
    '--ink-3': alpha(ink, dark ? 0.38 : 0.45),
    '--ink-4': alpha(ink, dark ? 0.16 : 0.18),
    '--fill-1': alpha(ink, dark ? 0.07 : 0.06),
    '--fill-2': alpha(ink, dark ? 0.12 : 0.1),
    '--fill-3': alpha(ink, dark ? 0.2 : 0.16),
    '--line': alpha(ink, dark ? 0.09 : 0.12),
    '--accent': accent,
    '--accent-soft': alpha(accent, dark ? 0.18 : 0.14),
    '--accent-ink': inkOn(accent),
    '--ok': theme.ok,
    '--warn': theme.warn,
    '--err': theme.err,
    '--rim': String(theme.rim),
    '--rim-color': alpha(mix(accent, ink, 0.35), 0.55),
    '--shadow': dark ? '0 18px 50px -12px rgba(0,0,0,0.55)' : '0 18px 50px -14px rgba(20,20,40,0.35)',
    '--font-ui': fontStack(opts.fontUi || theme.fontUi),
    '--font-display': fontStack(opts.fontDisplay || theme.fontDisplay),
    'color-scheme': theme.mode,
    ...lookVars(theme.look ?? {}),
  };
}

/** The look as CSS custom properties. A theme without one (an older custom theme) draws as before. */
function lookVars(l: Look): ThemeVars {
  return {
    '--surface': l.surface ?? 'none',
    '--edge': l.edge ?? '0 0 #0000',
    '--frame': l.frame ?? '0 0 #0000',
    '--outline': l.outline ?? 'transparent',
    '--round': String(Math.max(1, l.round ?? 1)),
    '--glow-a': String(l.glow ?? 0),
    '--text-glow': l.textGlow ?? 'none',
    '--display-style': l.italic ? 'italic' : 'normal',
    '--display-synthesis': l.trueWeights ? 'none' : 'weight style',
    '--tracking': l.tracking ?? 'normal',
    '--bar-h': l.bar?.height ?? '4px',
    '--bar-fill': l.bar?.fill ?? 'var(--ink)',
    '--solid-bg': l.solid?.bg ?? 'var(--ink)',
    '--solid-ink': l.solid?.ink ?? 'var(--island)',
    '--solid-ring': l.solid?.ring ?? '0 0 #0000',
  };
}

export function applyVars(el: HTMLElement, vars: ThemeVars): void {
  for (const [key, value] of Object.entries(vars)) el.style.setProperty(key, value);
}
