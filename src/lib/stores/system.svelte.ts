import { call, on, send } from '../bridge';
import type { Battery, QuotaProvider, Radios, Stats, Clip, Volume } from '../types';
import { settings } from './settings.svelte';

const HISTORY = 60;

class System {
  stats = $state<Stats | null>(null);
  cpu = $state<number[]>([]);
  memory = $state<number[]>([]);
  down = $state<number[]>([]);
  up = $state<number[]>([]);
  battery = $state<Battery | null>(null);
  quotas = $state<QuotaProvider[]>([]);
  clips = $state<Clip[]>([]);
  radios = $state<Radios | null>(null);
  volume = $state<Volume | null>(null);

  /** The highest plan usage across providers, for the resting island's ring. */
  topQuota = $derived.by(() => {
    let best: { provider: string; label: string; percent: number; resetsAt: number | null } | null = null;
    for (const p of this.quotas) {
      for (const item of p.items) {
        if (!best || item.percent > best.percent) best = { provider: p.provider, ...item };
      }
    }
    return best;
  });

  async start(onPower: (b: Battery | null, prev: Battery | null) => void, onVolume: (v: Volume) => void): Promise<void> {
    const push = (list: number[], value: number) => {
      const next = list.length >= HISTORY ? list.slice(list.length - HISTORY + 1) : list.slice();
      next.push(value);
      return next;
    };
    await on<Stats>('system://stats', (s) => {
      this.stats = s;
      this.cpu = push(this.cpu, s.cpu);
      this.memory = push(this.memory, s.memory);
      this.down = push(this.down, s.netDown);
      this.up = push(this.up, s.netUp);
      this.battery = s.battery;
    });
    await on<Battery | null>('power://changed', (b) => {
      const prev = this.battery;
      this.battery = b;
      onPower(b, prev);
    });
    await on<QuotaProvider[]>('quotas://update', (q) => (this.quotas = q));
    // A window opened after the last check (settings) asks for it.
    if (!this.quotas.length) this.quotas = (await call<QuotaProvider[]>('quotas_get').catch(() => null)) ?? [];
    await on<Clip[]>('clipboard://changed', (c) => (this.clips = c));
    await on<Volume>('volume://changed', (v) => {
      this.volume = v;
      onVolume(v);
    });
    this.battery = await call<Battery | null>('power_get').catch(() => null);
    this.clips = await call<Clip[]>('clipboard_list').catch(() => []);
    this.volume = await call<Volume>('volume_get').catch(() => null);
  }

  /** Use inside an $effect: readings flow while the returned cleanup has not run. */
  watch(): () => void {
    send('system_watch', { on: true });
    return () => send('system_watch', { on: false });
  }

  async refreshRadios(): Promise<void> {
    this.radios = await call<Radios>('radios_get').catch(() => null);
  }

  async setRadio(kind: 'wifi' | 'bluetooth', on: boolean): Promise<string | null> {
    try {
      this.radios = await call<Radios>('radio_set', { kind, on });
      return null;
    } catch (err) {
      return String(err);
    }
  }

  copy(text: string): void {
    send('clipboard_copy', { text });
  }

  setVolume(level: number): void {
    this.volume = { level, muted: this.volume?.muted ?? false };
    send('volume_set', { level });
  }

  toggleMute(): void {
    const muted = !(this.volume?.muted ?? false);
    this.volume = { level: this.volume?.level ?? 0, muted };
    send('volume_mute', { muted });
  }

  refreshQuotas(): void {
    if (settings.v.quotaClaude || settings.v.quotaCodex) send('quotas_refresh');
  }
}

export const system = new System();
