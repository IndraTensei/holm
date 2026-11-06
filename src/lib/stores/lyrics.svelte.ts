// Synced lyrics from LRCLIB (free, no key). Cached per track in localStorage.

import { getJson, inTauri } from '../bridge';
import { media } from './media.svelte';
import { settings } from './settings.svelte';

export interface LyricLine {
  time: number;
  text: string;
}

interface Entry {
  synced: LyricLine[];
  plain: string;
  instrumental: boolean;
}

const CACHE_KEY = 'tensei.lyrics.v1';
const MAX_CACHE = 150;

function parseLrc(lrc: string): LyricLine[] {
  const lines: LyricLine[] = [];
  for (const raw of lrc.split(/\r?\n/)) {
    const stamps = [...raw.matchAll(/\[(\d{1,2}):(\d{2})(?:[.:](\d{1,3}))?\]/g)];
    if (!stamps.length) continue;
    const text = raw.replace(/\[[^\]]*\]/g, '').trim();
    for (const m of stamps) {
      const frac = m[3] ? Number(m[3].padEnd(3, '0')) / 1000 : 0;
      lines.push({ time: Number(m[1]) * 60 + Number(m[2]) + frac, text });
    }
  }
  return lines.sort((a, b) => a.time - b.time);
}

function readCache(): Record<string, Entry> {
  try {
    return JSON.parse(localStorage.getItem(CACHE_KEY) ?? '{}');
  } catch {
    return {};
  }
}

function writeCache(cache: Record<string, Entry>): void {
  try {
    const keys = Object.keys(cache);
    if (keys.length > MAX_CACHE) for (const k of keys.slice(0, keys.length - MAX_CACHE)) delete cache[k];
    localStorage.setItem(CACHE_KEY, JSON.stringify(cache));
  } catch {
    /* storage full or unavailable: lyrics still work for this session */
  }
}

/** "Song Title (Official Video) [4K]" → "Song Title" for better matches. */
function cleanTitle(title: string): string {
  return title
    .replace(/\s*[([](official|lyric|lyrics|audio|video|visualizer|mv|hd|4k|remaster(ed)?)[^)\]]*[)\]]/gi, '')
    .replace(/\s*-\s*(official|lyric).*$/i, '')
    .trim();
}

class Lyrics {
  lines = $state<LyricLine[]>([]);
  plain = $state('');
  status = $state<'idle' | 'loading' | 'synced' | 'plain' | 'instrumental' | 'none'>('idle');
  private key = '';
  private cache: Record<string, Entry> | null = null;

  /** Index of the line being sung at `position` seconds, or -1. */
  index(position: number): number {
    const lines = this.lines;
    let lo = 0;
    let hi = lines.length - 1;
    let found = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (lines[mid].time <= position + 0.25) {
        found = mid;
        lo = mid + 1;
      } else hi = mid - 1;
    }
    return found;
  }

  /** Called from an $effect whenever the track changes. */
  async load(): Promise<void> {
    const now = media.now;
    if (!settings.v.lyrics || !now || !now.title) {
      this.key = '';
      this.lines = [];
      this.plain = '';
      this.status = 'idle';
      return;
    }
    const title = cleanTitle(now.title);
    const artist = now.artist.replace(/\s*-\s*Topic$/i, '');
    const key = `${artist}|${title}`.toLowerCase();
    if (key === this.key) return;
    this.key = key;
    this.cache ??= readCache();
    const hit = this.cache[key];
    if (hit) return this.apply(hit);

    this.status = 'loading';
    this.lines = [];
    this.plain = '';
    if (!inTauri) {
      const { SAMPLE_LYRICS } = await import('../mock');
      return this.apply({ synced: SAMPLE_LYRICS.map(([time, text]) => ({ time, text })), plain: '', instrumental: false });
    }
    const entry = await this.fetch(title, artist, now.album, now.duration);
    if (this.key !== key) return; // the track changed while we waited
    if (entry) {
      this.cache[key] = entry;
      writeCache(this.cache);
      this.apply(entry);
    } else this.status = 'none';
  }

  private apply(entry: Entry): void {
    this.lines = entry.synced;
    this.plain = entry.plain;
    this.status = entry.instrumental ? 'instrumental' : entry.synced.length ? 'synced' : entry.plain ? 'plain' : 'none';
  }

  private async fetch(title: string, artist: string, album: string, duration: number): Promise<Entry | null> {
    type Result = { syncedLyrics: string | null; plainLyrics: string | null; instrumental: boolean; duration: number };
    const toEntry = (r: Result): Entry => ({
      synced: r.syncedLyrics ? parseLrc(r.syncedLyrics) : [],
      plain: r.plainLyrics ?? '',
      instrumental: Boolean(r.instrumental),
    });
    try {
      const params = new URLSearchParams({ track_name: title, artist_name: artist });
      if (album) params.set('album_name', album);
      if (duration > 0) params.set('duration', String(Math.round(duration)));
      const exact = await getJson<Result>(`https://lrclib.net/api/get?${params}`);
      if (exact.status === 200 && exact.body) return toEntry(exact.body);
      const search = await getJson<Result[]>(`https://lrclib.net/api/search?${new URLSearchParams({ q: `${artist} ${title}` })}`);
      if (search.status !== 200 || !Array.isArray(search.body)) return null;
      const results = search.body;
      if (!results.length) return null;
      const best = results
        .filter((r) => r.syncedLyrics || r.plainLyrics)
        .sort((a, b) => Math.abs(a.duration - duration) - Math.abs(b.duration - duration))[0];
      return best ? toEntry(best) : null;
    } catch {
      return null;
    }
  }
}

export const lyrics = new Lyrics();
