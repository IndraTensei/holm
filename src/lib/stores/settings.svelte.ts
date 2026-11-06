import { call, on } from '../bridge';
import type { Theme } from '../themes';

export type PageId = 'home' | 'music' | 'agents' | 'system' | 'shelf' | 'focus';

export interface Settings {
  // Island
  style: 'notch' | 'pill' | 'line';
  monitor: number;
  offsetY: number;
  scale: number;
  openOn: 'hover' | 'click';
  hoverDelay: number;
  collapseDelay: number;
  hideInFullscreen: boolean;
  lineWhenMaximized: boolean;
  autostart: boolean;
  hotkey: string;
  pages: PageId[];
  // Look
  theme: string;
  customThemes: Theme[];
  fontUi: string;
  fontDisplay: string;
  adaptiveAccent: boolean;
  motion: 'spring' | 'smooth' | 'reduced';
  rim: boolean;
  // Resting island
  showClock: boolean;
  clock24h: boolean;
  showQuotaRing: boolean;
  compactTitle: 'none' | 'title' | 'lyrics';
  showBattery: boolean;
  // Music
  musicSource: 'auto' | 'pear' | 'system';
  pearHost: string;
  pearPort: number;
  pearToken: string;
  pearClientId: string;
  lyrics: boolean;
  trackToasts: boolean;
  volumeOsd: boolean;
  // Agents
  agents: boolean;
  hookPort: number;
  approvalTimeout: number;
  agentSounds: boolean;
  autoOpenApprovals: boolean;
  // Plan usage
  quotaClaude: boolean;
  quotaCodex: boolean;
  quotaMinutes: number;
  // Shelf and widgets
  clipboardHistory: boolean;
  clipboardMax: number;
  weather: boolean;
  weatherCity: string;
  weatherLat: number | null;
  weatherLon: number | null;
  units: 'metric' | 'imperial';
  focusMinutes: number;
  breakMinutes: number;
  onboarded: boolean;
}

export const DEFAULTS: Settings = {
  style: 'notch',
  monitor: 0,
  offsetY: 8,
  scale: 1,
  openOn: 'hover',
  hoverDelay: 140,
  collapseDelay: 380,
  hideInFullscreen: true,
  lineWhenMaximized: false,
  autostart: false,
  hotkey: 'Ctrl+Alt+Space',
  pages: ['home', 'music', 'agents', 'system', 'shelf', 'focus'],
  theme: 'obsidian',
  customThemes: [],
  fontUi: '',
  fontDisplay: '',
  adaptiveAccent: true,
  motion: 'spring',
  rim: true,
  showClock: true,
  clock24h: false,
  showQuotaRing: true,
  compactTitle: 'none',
  showBattery: true,
  musicSource: 'auto',
  pearHost: '127.0.0.1',
  pearPort: 26538,
  pearToken: '',
  pearClientId: '',
  lyrics: true,
  trackToasts: true,
  volumeOsd: true,
  agents: true,
  hookPort: 47821,
  approvalTimeout: 60,
  agentSounds: true,
  autoOpenApprovals: true,
  quotaClaude: true,
  quotaCodex: false,
  quotaMinutes: 5,
  clipboardHistory: true,
  clipboardMax: 30,
  weather: true,
  weatherCity: '',
  weatherLat: null,
  weatherLon: null,
  units: 'metric',
  focusMinutes: 25,
  breakMinutes: 5,
  onboarded: false,
};

class SettingsStore {
  v = $state<Settings>({ ...DEFAULTS });
  ready = $state(false);

  async load(): Promise<void> {
    const stored = await call<Partial<Settings> | null>('settings_get').catch(() => null);
    this.v = { ...DEFAULTS, ...(stored ?? {}) };
    this.ready = true;
    await on<Partial<Settings>>('settings://changed', (next) => {
      this.v = { ...DEFAULTS, ...next };
    });
  }

  async set(patch: Partial<Settings>): Promise<void> {
    this.v = { ...this.v, ...patch };
    await call('settings_patch', { patch }).catch((err) => console.warn('[island] settings', err));
  }
}

export const settings = new SettingsStore();
