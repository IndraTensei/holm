// Pear Desktop (the open-source YouTube Music desktop app, github.com/pear-devs/pear-desktop)
// through its "API Server" plugin: REST for actions, a WebSocket for live state. It adds what
// Windows media controls cannot carry: like/dislike, Pear's own volume, repeat and shuffle state.

import { settings } from './settings.svelte';

export interface PearSong {
  title: string;
  artist: string;
  album: string;
  art: string | null;
  duration: number;
  videoId: string;
  url: string;
}

type Status = 'off' | 'connecting' | 'needs-auth' | 'denied' | 'connected' | 'unreachable';
type Like = 'LIKE' | 'DISLIKE' | 'INDIFFERENT' | null;

interface RawSong {
  title?: string;
  artist?: string;
  album?: string | null;
  imageSrc?: string | null;
  songDuration?: number;
  videoId?: string;
  url?: string;
}

function mapSong(raw: RawSong | undefined): PearSong | null {
  if (!raw?.title) return null;
  return {
    title: raw.title,
    artist: raw.artist ?? '',
    album: raw.album ?? '',
    art: raw.imageSrc ?? null,
    duration: raw.songDuration ?? 0,
    videoId: raw.videoId ?? '',
    url: raw.url ?? '',
  };
}

class Pear {
  status = $state<Status>('off');
  song = $state<PearSong | null>(null);
  playing = $state(false);
  position = $state(0);
  positionAt = $state(0);
  volume = $state(100);
  muted = $state(false);
  repeat = $state<'NONE' | 'ALL' | 'ONE'>('NONE');
  shuffle = $state(false);
  like = $state<Like>(null);

  private ws: WebSocket | null = null;
  private retryMs = 4000;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private running = false;

  private get base(): string {
    return `http://${settings.v.pearHost}:${settings.v.pearPort}`;
  }

  private headers(json = false): HeadersInit {
    const h: Record<string, string> = {};
    if (settings.v.pearToken) h.Authorization = `Bearer ${settings.v.pearToken}`;
    if (json) h['Content-Type'] = 'application/json';
    return h;
  }

  get connected(): boolean {
    return this.status === 'connected';
  }

  livePosition(now: number): number {
    const p = this.playing ? this.position + (now - this.positionAt) / 1000 : this.position;
    const d = this.song?.duration ?? 0;
    return d > 0 ? Math.min(Math.max(0, p), d) : Math.max(0, p);
  }

  /** Starts probing for Pear; safe to call repeatedly. */
  ensureRunning(): void {
    if (this.running) return;
    this.running = true;
    void this.connect();
  }

  stop(): void {
    this.running = false;
    if (this.timer) clearTimeout(this.timer);
    this.ws?.close();
    this.ws = null;
    this.status = 'off';
  }

  private schedule(): void {
    if (!this.running) return;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.connect(), this.retryMs);
    this.retryMs = Math.min(this.retryMs * 1.6, 30000);
  }

  async connect(): Promise<void> {
    if (this.ws) return;
    if (this.status !== 'connected') this.status = 'connecting';
    try {
      const res = await fetch(`${this.base}/api/v1/song`, { headers: this.headers() });
      if (res.status === 401 || res.status === 403) {
        this.status = 'needs-auth';
        return; // waits for authorize(), started from the island or settings
      }
      if (!res.ok && res.status !== 204) throw new Error(`HTTP ${res.status}`);
      this.openSocket();
    } catch {
      this.status = 'unreachable';
      this.schedule();
    }
  }

  /** Asks Pear for access; Pear shows an Allow/Deny dialog the first time. */
  async authorize(): Promise<void> {
    let id = settings.v.pearClientId;
    if (!id) {
      id = `holm-${Math.random().toString(36).slice(2, 8)}`;
      await settings.set({ pearClientId: id });
    }
    this.status = 'connecting';
    try {
      const res = await fetch(`${this.base}/auth/${encodeURIComponent(id)}`, { method: 'POST' });
      if (res.status === 403) {
        this.status = 'denied';
        return;
      }
      const body = (await res.json()) as { accessToken?: string };
      if (!body.accessToken) throw new Error('no token');
      await settings.set({ pearToken: body.accessToken });
      this.running = true;
      this.retryMs = 4000;
      await this.connect();
    } catch {
      this.status = 'unreachable';
    }
  }

  private openSocket(): void {
    const token = settings.v.pearToken ? `?token=${encodeURIComponent(settings.v.pearToken)}` : '';
    const ws = new WebSocket(`ws://${settings.v.pearHost}:${settings.v.pearPort}/api/v1/ws${token}`);
    this.ws = ws;
    ws.onopen = () => {
      this.status = 'connected';
      this.retryMs = 4000;
    };
    ws.onmessage = (e) => {
      try {
        this.handle(JSON.parse(String(e.data)));
      } catch {
        /* ignore malformed frames */
      }
    };
    ws.onclose = (e) => {
      this.ws = null;
      this.playing = false;
      if (e.code === 1008) {
        this.status = 'needs-auth';
        return;
      }
      this.status = 'unreachable';
      this.schedule();
    };
  }

  private handle(msg: Record<string, unknown> & { type: string }): void {
    const now = Date.now();
    const position = typeof msg.position === 'number' ? msg.position : undefined;
    switch (msg.type) {
      case 'PLAYER_INFO':
        this.song = mapSong(msg.song as RawSong);
        this.playing = Boolean(msg.isPlaying);
        this.volume = Number(msg.volume ?? 100);
        this.muted = Boolean(msg.muted);
        this.repeat = (msg.repeat as Pear['repeat']) ?? 'NONE';
        this.shuffle = Boolean(msg.shuffle);
        void this.fetchLike();
        break;
      case 'VIDEO_CHANGED':
        this.song = mapSong(msg.song as RawSong);
        this.like = null;
        setTimeout(() => void this.fetchLike(), 600);
        break;
      case 'PLAYER_STATE_CHANGED':
        this.playing = Boolean(msg.isPlaying);
        break;
      case 'VOLUME_CHANGED':
        this.volume = Number(msg.volume ?? this.volume);
        this.muted = Boolean(msg.muted);
        break;
      case 'REPEAT_CHANGED':
        this.repeat = (msg.repeat as Pear['repeat']) ?? this.repeat;
        break;
      case 'SHUFFLE_CHANGED':
        this.shuffle = Boolean(msg.shuffle);
        break;
    }
    if (position !== undefined) {
      this.position = position;
      this.positionAt = now;
    }
  }

  private async post(path: string, body?: unknown): Promise<void> {
    try {
      await fetch(`${this.base}/api/v1/${path}`, {
        method: 'POST',
        headers: this.headers(body !== undefined),
        body: body === undefined ? undefined : JSON.stringify(body),
      });
    } catch (err) {
      console.warn('[pear]', path, err);
    }
  }

  async fetchLike(): Promise<void> {
    try {
      const res = await fetch(`${this.base}/api/v1/like-state`, { headers: this.headers() });
      if (res.ok) this.like = ((await res.json()) as { state: Like }).state ?? null;
    } catch {
      /* Pear went away; the socket will notice */
    }
  }

  toggle = () => {
    this.playing = !this.playing;
    this.position = this.livePosition(Date.now());
    this.positionAt = Date.now();
    return this.post('toggle-play');
  };
  next = () => this.post('next');
  prev = () => this.post('previous');
  seek = (seconds: number) => {
    this.position = seconds;
    this.positionAt = Date.now();
    return this.post('seek-to', { seconds });
  };
  toggleLike = async () => {
    this.like = this.like === 'LIKE' ? 'INDIFFERENT' : 'LIKE';
    await this.post('like');
    setTimeout(() => void this.fetchLike(), 400);
  };
  toggleDislike = async () => {
    this.like = this.like === 'DISLIKE' ? 'INDIFFERENT' : 'DISLIKE';
    await this.post('dislike');
    setTimeout(() => void this.fetchLike(), 400);
  };
  toggleShuffle = () => this.post('shuffle');
  cycleRepeat = () => this.post('switch-repeat', { iteration: 1 });
  setVolume = (volume: number) => {
    this.volume = volume;
    return this.post('volume', { volume: Math.round(volume) });
  };
  toggleMute = () => this.post('toggle-mute');
}

export const pear = new Pear();
