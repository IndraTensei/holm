// Themes: a palette plus a type pairing. The island itself stays a solid object (like the
// hardware it imitates); themes change its fill, ink, accent and the rim light around it.

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
}

const base = { ok: '#5ce39b', warn: '#ffc35a', err: '#ff6b6b', rim: 0.35 };

export const THEMES: Theme[] = [
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
  };
}

export function applyVars(el: HTMLElement, vars: ThemeVars): void {
  for (const [key, value] of Object.entries(vars)) el.style.setProperty(key, value);
}
