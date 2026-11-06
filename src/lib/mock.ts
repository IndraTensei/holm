// Browser-only stand-in for the Rust side, so the UI can be designed and screenshotted
// without Windows. Never loaded inside the app.

import { DEFAULTS } from './stores/settings.svelte';
import type { AgentsSnapshot, MediaState, QuotaProvider, Stats } from './types';

type Handler = (payload: unknown) => void;
const handlers = new Map<string, Set<Handler>>();
const params = new URLSearchParams(location.search);

function emit(event: string, payload: unknown): void {
  handlers.get(event)?.forEach((h) => h(payload));
}

// `?film=1`: scripts/film.py plays events into the preview to record the promo clips.
if (params.has('film')) Object.assign(window, { __film: { emit } });

function cover(a: string, b: string, c: string): string {
  const svg = `<svg xmlns='http://www.w3.org/2000/svg' width='120' height='120'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='${a}'/><stop offset='1' stop-color='${b}'/></linearGradient></defs><rect width='120' height='120' fill='url(#g)'/><circle cx='84' cy='38' r='22' fill='${c}' opacity='0.85'/><rect x='14' y='78' width='70' height='8' rx='4' fill='white' opacity='0.5'/><rect x='14' y='92' width='44' height='8' rx='4' fill='white' opacity='0.3'/></svg>`;
  return `data:image/svg+xml;base64,${btoa(svg)}`;
}

const now = Date.now();
const settingsState: Record<string, unknown> = {
  ...DEFAULTS,
  weatherCity: 'Navi Mumbai',
  weatherLat: 19.03,
  weatherLon: 73.03,
  theme: params.get('theme') ?? DEFAULTS.theme,
  style: params.get('style') ?? DEFAULTS.style,
  compactTitle: params.get('title') ?? DEFAULTS.compactTitle,
};

const playing = params.get('music') !== 'off';
const media: MediaState = {
  current: 'Spotify.exe',
  sessions: playing
    ? [
        {
          id: 'Spotify.exe',
          app: 'Spotify',
          title: params.get('track') ?? 'Lanterns Over Kochi',
          artist: 'The Monsoon Office',
          album: 'Backwater Radio',
          art: cover('#ff7a59', '#6a3df0', '#ffd166'),
          status: 'playing',
          position: 71,
          duration: 214,
          updatedAt: now,
          canPlayPause: true,
          canNext: true,
          canPrev: true,
          canSeek: true,
          shuffle: false,
          repeat: 'none',
        },
        {
          id: 'chrome',
          app: 'Chrome',
          title: 'Grand finals: day two',
          artist: 'Esports broadcast',
          album: '',
          art: cover('#0f3d3e', '#12c2e9', '#c471ed'),
          status: 'paused',
          position: 1200,
          duration: 0,
          updatedAt: now,
          canPlayPause: true,
          canNext: false,
          canPrev: false,
          canSeek: false,
          shuffle: null,
          repeat: null,
        },
      ]
    : [],
};

const agentsState: AgentsSnapshot = {
  sessions:
    params.get('agents') === 'off'
      ? []
      : [
          {
            id: 's1',
            agent: 'claude',
            project: 'qinshin-bot',
            cwd: 'D:\\code\\qinshin-bot',
            status: params.get('approval') ? 'waiting' : 'working',
            activity: { verb: 'Editing', target: 'commands.ts', tool: 'Edit', kind: 'edit' },
            prompt: 'Add a /standings command that reads the league table',
            message: '',
            turnStartedAt: now - 154_000,
            updatedAt: now,
            tools: 14,
            edited: ['commands.ts', 'league.ts'],
            subagents: 0,
            log: [
              { at: now - 5_000, kind: 'edit', text: 'Editing commands.ts' },
              { at: now - 22_000, kind: 'run', text: 'Running npm test -- league' },
              { at: now - 48_000, kind: 'read', text: 'Reading league.ts' },
              { at: now - 71_000, kind: 'search', text: 'Searching standings' },
              { at: now - 154_000, kind: 'prompt', text: 'Add a /standings command that reads the league table' },
            ],
          },
          {
            id: 's2',
            agent: 'claude',
            project: 'meetbot',
            cwd: 'D:\\code\\meetbot',
            status: 'done',
            activity: null,
            prompt: 'Store transcripts per meeting',
            message:
              'Transcripts are now saved per meeting in the `transcripts` table, keyed by meeting id. I added a migration and two tests; both pass.',
            turnStartedAt: now - 900_000,
            updatedAt: now - 420_000,
            tools: 31,
            edited: ['db.ts', 'migrations/004_transcripts.sql', 'transcripts.test.ts'],
            subagents: 0,
            log: [{ at: now - 420_000, kind: 'done', text: 'Finished' }],
          },
        ],
  approvals: params.get('approval')
    ? [
        {
          id: 'a1',
          sessionId: 's1',
          project: 'qinshin-bot',
          tool: 'Bash',
          title: 'Running npm install discord.js@latest',
          detail: 'npm install discord.js@latest',
          createdAt: now,
          expiresAt: now + 60_000,
        },
      ]
    : [],
};

const quotas: QuotaProvider[] = [
  {
    provider: 'claude',
    ok: true,
    plan: 'max',
    error: '',
    fetchedAt: now,
    items: [
      { label: '5-hour', percent: 64, resetsAt: now + 98 * 60_000, windowMinutes: 300 },
      { label: 'Weekly', percent: 31, resetsAt: now + 3.2 * 86_400_000, windowMinutes: 10080 },
    ],
  },
];

function stats(): Stats {
  const t = Date.now() / 1000;
  return {
    cpu: 18 + 12 * Math.sin(t / 3) + Math.random() * 6,
    memory: 61 + Math.sin(t / 20) * 2,
    memoryUsedGb: 9.8,
    memoryTotalGb: 16,
    netDown: 240_000 + Math.random() * 900_000,
    netUp: 40_000 + Math.random() * 60_000,
    battery: { percent: 78, charging: false, plugged: true, saver: false, secondsLeft: null },
  };
}

setInterval(() => emit('system://stats', stats()), 1000);

export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const result = ((): unknown => {
    switch (cmd) {
      case 'settings_get':
        return settingsState;
      case 'settings_patch':
        Object.assign(settingsState, args?.patch as object);
        setTimeout(() => emit('settings://changed', { ...settingsState }), 0);
        return settingsState;
      case 'media_refresh':
        setTimeout(() => emit('media://state', media), 0);
        return null;
      case 'media_control': {
        const s = media.sessions[0];
        if (s && args?.action === 'toggle') {
          s.position = s.position + (s.status === 'playing' ? (Date.now() - s.updatedAt) / 1000 : 0);
          s.updatedAt = Date.now();
          s.status = s.status === 'playing' ? 'paused' : 'playing';
          emit('media://state', { ...media, sessions: [...media.sessions] });
        }
        return null;
      }
      case 'agents_state':
        return agentsState;
      case 'power_get':
        return stats().battery;
      case 'clipboard_list':
        return [
          { id: 1, at: now - 60_000, text: 'https://github.com/pear-devs/pear-desktop' },
          { id: 2, at: now - 300_000, text: 'npm run tauri build -- --bundles nsis' },
          { id: 3, at: now - 900_000, text: 'Match 3 starts at 19:30 IST, lobby code 4471' },
        ];
      case 'volume_get':
        return { level: 0.42, muted: false };
      case 'radios_get':
        return { allowed: true, wifi: true, bluetooth: false, ssid: 'Tensei-5G', signal: 82 };
      case 'data_get':
        if (args?.key === 'note') return 'Ask Vizta about the scrim schedule\nShip island v0.1';
        if (args?.key === 'shelf')
          return [
            { path: 'C:\\Users\\ashish\\Downloads\\bracket-final.png', addedAt: now },
            { path: 'C:\\Users\\ashish\\Documents\\invoice-3nttb.pdf', addedAt: now },
            { path: 'C:\\Users\\ashish\\Desktop\\overlay.psd', addedAt: now },
          ];
        return null;
      case 'path_exists':
        return (args?.paths as string[]).map(() => true);
      case 'hooks_status':
        return { installed: false, events: 0, path: 'C:\\Users\\ashish\\.claude\\settings.json', claudeFound: true, port: 47821, mode: '' };
      case 'monitors_list':
        return [{ index: 0, name: '\\\\.\\DISPLAY1', primary: true, width: 2560, height: 1440, scale: 1.25 }];
      case 'autostart_get':
        return false;
      case 'island_cursor':
        return null;
      case 'island_state':
        return { maximized: false, app: '', hidden: false, shown: true };
      case 'app_volume_get':
        return { level: 0.7, muted: false };
      case 'app_volume_set':
      case 'app_volume_mute':
        return true;
      case 'quotas_get':
        return [];
      case 'harness_scan': {
        const now = Date.now();
        return [
          { id: 'claude', name: 'Claude Code', installed: true, running: true, lastUsed: now - 20_000, today: { tokens: 1_284_000, sessions: 3, cost: null }, plan: true },
          { id: 'codex', name: 'Codex', installed: true, running: false, lastUsed: now - 3 * 3600_000, today: { tokens: 412_500, sessions: 2, cost: null }, plan: true },
          { id: 'opencode', name: 'OpenCode', installed: true, running: false, lastUsed: now - 26 * 3600_000, today: null, plan: false },
          { id: 'antigravity', name: 'Antigravity', installed: true, running: false, lastUsed: now - 50 * 60_000, today: { tokens: null, sessions: 1, cost: null }, plan: false },
          { id: 'cline', name: 'Cline', installed: true, running: false, lastUsed: now - 9 * 86400_000, today: null, plan: false },
          { id: 'copilot', name: 'GitHub Copilot CLI', installed: true, running: false, lastUsed: now - 4 * 86400_000, today: null, plan: false },
          { id: 'gemini', name: 'Gemini CLI', installed: false, running: false, lastUsed: null, today: null, plan: false },
          { id: 'cursor', name: 'Cursor Agent', installed: false, running: false, lastUsed: null, today: null, plan: false },
          { id: 'amp', name: 'Amp', installed: false, running: false, lastUsed: null, today: null, plan: false },
        ];
      }
      case 'quiet_get':
        return false;
      case 'island_summon':
      case 'island_set_hidden':
        return null;
      default:
        return null;
    }
  })();
  if (cmd === 'agents_state') setTimeout(() => emit('quotas://update', quotas), 50);
  return result as T;
}

export async function on<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  const set = handlers.get(event) ?? new Set();
  set.add(handler as Handler);
  handlers.set(event, set);
  return () => set.delete(handler as Handler);
}

/** Original sample lyrics for previews (LRCLIB is used in the app). */
export const SAMPLE_LYRICS = [
  [60, 'Lanterns on the water, slow and gold'],
  [66, 'Every ferry carries something old'],
  [72, 'I keep the radio low so I can hear'],
  [78, 'The backwater humming that you are near'],
  [85, 'Hold on, hold on, the tide is turning'],
  [92, 'Hold on, the city lights are burning'],
] as const;
