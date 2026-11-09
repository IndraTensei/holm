// A game's colour and monogram, for the intro and the end-of-game card when there's no logo.

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

export function hueOf(name: string): number {
  const known = HUES[name.toLowerCase()];
  if (known !== undefined) return known;
  let h = 0;
  for (const c of name) h = (h * 31 + c.charCodeAt(0)) % 360;
  return h;
}

const SMALL = new Set(['of', 'the', 'a', 'an', 'and', 'in', 'on', 'to']);

/** "Grand Theft Auto V" → "GV", "Red Dead Redemption 2" → "R2", "Hades II" → "HII", "Valorant" → "V". */
export function monogram(name: string): string {
  const words = name
    .split(/[\s\-:_.]+/)
    .filter(Boolean)
    .filter((w, i) => i === 0 || !SMALL.has(w.toLowerCase()));
  if (words.length === 0) return '?';
  const first = words[0][0].toUpperCase();
  if (words.length === 1) return first;
  const last = words[words.length - 1];
  // A sequel number or numeral reads as itself: R2, HII, C2.
  if (/^(\d{1,4}|[IVX]{1,4})$/i.test(last)) return first + last.toUpperCase();
  return first + last[0].toUpperCase();
}
