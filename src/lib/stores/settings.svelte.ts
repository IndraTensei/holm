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
  /** Over a maximized window: learn per app, always a thin line, or never. */
  overMaximized: 'learn' | 'line' | 'full';
  /** Apps (lower-case program names) the island rests as a line over; learned or added. */
  lineApps: string[];
  /** Browsers rest as a line over a maximized window from the start (their tabs sit under the island). */
  browsersAsLine: boolean;
  /** Rest as a thin line when nothing is happening. */
  tuckWhenIdle: boolean;
  /** Seconds of nothing before tucking. */
  tuckAfter: number;
  /** Hold pop-ups and chimes while Windows' Do not disturb is on. */
  respectDnd: boolean;
  autostart: boolean;
  /** WebView2 compositing. 'software' uses ~60 MB less memory; read by Rust at start-up. */
  graphics: 'gpu' | 'software';
  hotkey: string;
  /** Hides the island, and shows it again. */
  hideHotkey: string;
  // Games
  /** "Opening Valorant" when a game comes to the front. */
  gameIntro: boolean;
  /** Step aside while a game is in front, even windowed. Read by Rust. */
  smartHide: boolean;
  /** Games that are not recognised on their own. Read by Rust. */
  customGames: { exe: string; name: string }[];
  /** Say how long you played when a game closes. */
  gameRecap: boolean;
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
  /** AI tools shown on the Agents page, by id. Claude Code and Codex use quotaClaude and quotaCodex. */
  toolsShown: Record<string, boolean>;
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
  hoverDelay: 220,
  collapseDelay: 380,
  hideInFullscreen: true,
  overMaximized: 'learn',
  lineApps: [],
  browsersAsLine: true,
  tuckWhenIdle: true,
  tuckAfter: 20,
  respectDnd: true,
  autostart: false,
  graphics: 'software',
  hotkey: 'Ctrl+Alt+Space',
  hideHotkey: 'Ctrl+Alt+H',
  gameIntro: true,
  smartHide: true,
  customGames: [],
  gameRecap: true,
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
  trackToasts: false,
  volumeOsd: true,
  agents: true,
  hookPort: 47821,
  approvalTimeout: 60,
  agentSounds: true,
  autoOpenApprovals: true,
  quotaClaude: true,
  quotaCodex: true,
  quotaMinutes: 1,
  toolsShown: {},
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
