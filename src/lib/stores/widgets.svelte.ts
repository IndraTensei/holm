// Shelf (files kept on the island), the quick note, the focus timer and weather.

import { call, getJson, send } from '../bridge';
import { settings } from './settings.svelte';

export interface ShelfItem {
  path: string;
  addedAt: number;
}

class Shelf {
  items = $state<ShelfItem[]>([]);

  async load(): Promise<void> {
    const saved = await call<ShelfItem[] | null>('data_get', { key: 'shelf' }).catch(() => null);
    if (!saved?.length) return;
    const exists = await call<boolean[]>('path_exists', { paths: saved.map((s) => s.path) }).catch(() => saved.map(() => true));
    this.items = saved.filter((_, i) => exists[i]);
  }

  private save(): void {
    send('data_set', { key: 'shelf', value: this.items });
  }

  add(paths: string[]): number {
    const fresh = paths.filter((p) => !this.items.some((i) => i.path === p));
    this.items = [...fresh.map((path) => ({ path, addedAt: Date.now() })), ...this.items].slice(0, 40);
    this.save();
    return fresh.length;
  }

  remove(path: string): void {
    this.items = this.items.filter((i) => i.path !== path);
    this.save();
  }

  clear(): void {
    this.items = [];
    this.save();
  }

  open = (path: string) => send('open_path', { path });
  reveal = (path: string) => send('reveal_path', { path });
}

class Note {
  text = $state('');
  private timer: ReturnType<typeof setTimeout> | null = null;

  async load(): Promise<void> {
    this.text = (await call<string | null>('data_get', { key: 'note' }).catch(() => null)) ?? '';
  }

  update(text: string): void {
    this.text = text;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => send('data_set', { key: 'note', value: this.text }), 500);
  }
}

type Phase = 'idle' | 'focus' | 'break' | 'paused';

class Focus {
  phase = $state<Phase>('idle');
  /** When the running phase ends (ms), or remaining ms while paused. */
  endsAt = $state(0);
  remainingWhenPaused = $state(0);
  pausedPhase = $state<'focus' | 'break'>('focus');
  totalMs = $state(0);
  sessionsToday = $state(0);
  onFinish: ((phase: 'focus' | 'break') => void) | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;

  running = $derived(this.phase === 'focus' || this.phase === 'break');

  remaining(now: number): number {
    if (this.phase === 'paused') return this.remainingWhenPaused;
    if (!this.running) return settings.v.focusMinutes * 60_000;
    return Math.max(0, this.endsAt - now);
  }

  progress(now: number): number {
    if (!this.totalMs) return 0;
    return 1 - this.remaining(now) / this.totalMs;
  }

  start(phase: 'focus' | 'break' = 'focus', minutes?: number): void {
    const m = minutes ?? (phase === 'focus' ? settings.v.focusMinutes : settings.v.breakMinutes);
    this.totalMs = m * 60_000;
    this.endsAt = Date.now() + this.totalMs;
    this.phase = phase;
    this.arm();
  }

  pause(): void {
    if (!this.running) return;
    this.remainingWhenPaused = Math.max(0, this.endsAt - Date.now());
    this.pausedPhase = this.phase as 'focus' | 'break';
    this.phase = 'paused';
    if (this.timer) clearTimeout(this.timer);
  }

  resume(): void {
    if (this.phase !== 'paused') return;
    this.endsAt = Date.now() + this.remainingWhenPaused;
    this.phase = this.pausedPhase;
    this.arm();
  }

  reset(): void {
    this.phase = 'idle';
    this.totalMs = 0;
    if (this.timer) clearTimeout(this.timer);
  }

  private arm(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      const finished = this.phase as 'focus' | 'break';
      if (finished === 'focus') this.sessionsToday++;
      this.phase = 'idle';
      this.totalMs = 0;
      this.onFinish?.(finished);
    }, Math.max(0, this.endsAt - Date.now()));
  }
}

export interface Weather {
  temp: number;
  code: number;
  isDay: boolean;
  high: number;
  low: number;
  city: string;
  at: number;
}

class WeatherStore {
  now = $state<Weather | null>(null);
  private timer: ReturnType<typeof setInterval> | null = null;

  start(): void {
    void this.refresh();
    this.timer ??= setInterval(() => void this.refresh(), 30 * 60_000);
  }

  async refresh(): Promise<void> {
    const { weather, weatherLat: lat, weatherLon: lon, units, weatherCity } = settings.v;
    if (!weather || lat == null || lon == null) {
      this.now = null;
      return;
    }
    try {
      const unit = units === 'imperial' ? '&temperature_unit=fahrenheit' : '';
      const { body: d } = await getJson<any>(
        `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code,is_day&daily=temperature_2m_max,temperature_2m_min&forecast_days=1&timezone=auto${unit}`,
      );
      if (!d?.current) return;
      this.now = {
        temp: d.current.temperature_2m,
        code: d.current.weather_code,
        isDay: d.current.is_day === 1,
        high: d.daily.temperature_2m_max[0],
        low: d.daily.temperature_2m_min[0],
        city: weatherCity,
        at: Date.now(),
      };
    } catch {
      /* keep the last reading */
    }
  }

  static async search(name: string): Promise<{ name: string; country: string; admin: string; lat: number; lon: number }[]> {
    const { body: d } = await getJson<any>(`https://geocoding-api.open-meteo.com/v1/search?count=6&name=${encodeURIComponent(name)}`);
    return (d?.results ?? []).map((r: { name: string; country: string; admin1?: string; latitude: number; longitude: number }) => ({
      name: r.name,
      country: r.country,
      admin: r.admin1 ?? '',
      lat: r.latitude,
      lon: r.longitude,
    }));
  }
}

/** WMO weather code → a short word and an icon family. */
export function describeWeather(code: number): { label: string; icon: 'sun' | 'cloud-sun' | 'cloud' | 'fog' | 'rain' | 'snow' | 'storm' } {
  if (code === 0) return { label: 'Clear', icon: 'sun' };
  if (code <= 2) return { label: 'Partly cloudy', icon: 'cloud-sun' };
  if (code === 3) return { label: 'Cloudy', icon: 'cloud' };
  if (code <= 48) return { label: 'Fog', icon: 'fog' };
  if (code <= 67 || (code >= 80 && code <= 82)) return { label: 'Rain', icon: 'rain' };
  if (code <= 77 || code === 85 || code === 86) return { label: 'Snow', icon: 'snow' };
  return { label: 'Thunderstorm', icon: 'storm' };
}

export const shelf = new Shelf();
export const note = new Note();
export const focus = new Focus();
export const weather = new WeatherStore();
export const searchCity = WeatherStore.search;
