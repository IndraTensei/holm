// One "now playing" from every source: Windows media sessions plus Pear's richer API.

import { call, on, send } from '../bridge';
import type { MediaState, SmtcSession } from '../types';
import { pear } from './pear.svelte';
import { settings } from './settings.svelte';

export type Repeat = 'none' | 'list' | 'track' | null;

export interface NowPlaying {
  key: string;
  source: 'pear' | 'smtc';
  sessionId: string;
  app: string;
  title: string;
  artist: string;
  album: string;
  art: string | null;
  playing: boolean;
  duration: number;
  canSeek: boolean;
  canNext: boolean;
  canPrev: boolean;
  shuffle: boolean | null;
  repeat: Repeat;
  like: 'LIKE' | 'DISLIKE' | 'INDIFFERENT' | null;
  /** Pear only: Pear's own volume 0–100. */
  volume: number | null;
}

const PEAR_ID = /youtube-?music|pear/i;

function fromSmtc(s: SmtcSession): NowPlaying {
  return {
    key: `${s.id}|${s.title}|${s.artist}`,
    source: 'smtc',
    sessionId: s.id,
    app: s.app,
    title: s.title,
    artist: s.artist,
    album: s.album,
    art: s.art,
    playing: s.status === 'playing',
    duration: s.duration,
    canSeek: s.canSeek && s.duration > 0,
    canNext: s.canNext,
    canPrev: s.canPrev,
    shuffle: s.shuffle,
    repeat: s.repeat,
    like: null,
    volume: null,
  };
}

function fromPear(smtcArt: string | null): NowPlaying | null {
  const song = pear.song;
  if (!song) return null;
  const repeat: Repeat = pear.repeat === 'ONE' ? 'track' : pear.repeat === 'ALL' ? 'list' : 'none';
  return {
    key: `pear|${song.videoId || song.title}`,
    source: 'pear',
    sessionId: 'pear',
    app: 'Pear',
    title: song.title,
    artist: song.artist,
    album: song.album,
    // Pear's art is a large remote image; fall back to the session thumbnail.
    art: song.art ?? smtcArt,
    playing: pear.playing,
    duration: song.duration,
    canSeek: song.duration > 0,
    canNext: true,
    canPrev: true,
    shuffle: pear.shuffle,
    repeat,
    like: pear.like,
    volume: pear.muted ? 0 : pear.volume,
  };
}

class Media {
  sessions = $state<SmtcSession[]>([]);
  current = $state<string | null>(null);
  /** A source the user picked in the music page; null follows whatever plays. */
  pinned = $state<string | null>(null);
  private lastPlaying = new Map<string, number>();

  options = $derived.by((): NowPlaying[] => {
    const usePear = settings.v.musicSource !== 'system' && pear.connected && pear.song;
    const pearSession = this.sessions.find((s) => PEAR_ID.test(s.id));
    const list: NowPlaying[] = [];
    if (usePear) {
      const p = fromPear(pearSession?.art ?? null);
      if (p) list.push(p);
    }
    if (settings.v.musicSource !== 'pear') {
      for (const s of this.sessions) {
        if (!s.title) continue;
        if (usePear && PEAR_ID.test(s.id)) continue;
        list.push(fromSmtc(s));
      }
    }
    return list;
  });

  now = $derived.by((): NowPlaying | null => {
    const list = this.options;
    if (list.length === 0) return null;
    if (this.pinned) {
      const pinned = list.find((o) => o.sessionId === this.pinned);
      if (pinned) return pinned;
    }
    const playing = list.filter((o) => o.playing);
    if (playing.length) {
      return playing.find((o) => o.source === 'pear') ?? playing.find((o) => o.sessionId === this.current) ?? playing[0];
    }
    return list.find((o) => o.sessionId === this.current) ?? list[0];
  });

  /** Music counts as live while playing and for five minutes after it pauses. */
  live(now: number): boolean {
    const n = this.now;
    if (!n) return false;
    if (n.playing) {
      this.lastPlaying.set(n.key, now);
      return true;
    }
    return now - (this.lastPlaying.get(n.key) ?? 0) < 5 * 60_000;
  }

  position(now: number): number {
    const n = this.now;
    if (!n) return 0;
    if (n.source === 'pear') return pear.livePosition(now);
    const s = this.sessions.find((x) => x.id === n.sessionId);
    if (!s) return 0;
    const p = s.status === 'playing' ? s.position + (now - s.updatedAt) / 1000 : s.position;
    return s.duration > 0 ? Math.min(Math.max(0, p), s.duration) : Math.max(0, p);
  }

  async start(): Promise<void> {
    await on<MediaState>('media://state', (state) => {
      this.sessions = state.sessions;
      this.current = state.current;
      if (settings.v.musicSource !== 'system' && (state.sessions.some((s) => PEAR_ID.test(s.id)) || settings.v.pearToken)) {
        pear.ensureRunning();
      }
    });
    if (settings.v.musicSource === 'pear' || settings.v.pearToken) pear.ensureRunning();
    send('media_refresh');
  }

  private smtc(action: string, value?: number): void {
    const n = this.now;
    send('media_control', { session: n?.sessionId ?? null, action, value: value ?? null });
  }

  toggle = () => (this.now?.source === 'pear' ? void pear.toggle() : this.smtc('toggle'));
  next = () => (this.now?.source === 'pear' ? void pear.next() : this.smtc('next'));
  prev = () => (this.now?.source === 'pear' ? void pear.prev() : this.smtc('prev'));
  seek = (seconds: number) => (this.now?.source === 'pear' ? void pear.seek(seconds) : this.smtc('seek', seconds));
  shuffle = () => (this.now?.source === 'pear' ? void pear.toggleShuffle() : this.smtc('shuffle'));
  repeat = () => (this.now?.source === 'pear' ? void pear.cycleRepeat() : this.smtc('repeat'));
  like = () => void pear.toggleLike();
  dislike = () => void pear.toggleDislike();

  async volume(): Promise<{ level: number; muted: boolean }> {
    return call('volume_get');
  }
}

export const media = new Media();
