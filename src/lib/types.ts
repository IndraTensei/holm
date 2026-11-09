export type Live = 'music' | 'agent' | 'timer' | 'idle';

// Shapes of what the Rust side emits. Keep in sync with src-tauri/src/*.rs (serde camelCase).

export interface SmtcSession {
  id: string;
  app: string;
  title: string;
  artist: string;
  album: string;
  art: string | null;
  status: 'playing' | 'paused' | 'stopped' | 'changing' | 'opened' | 'closed';
  position: number;
  duration: number;
  updatedAt: number;
  canPlayPause: boolean;
  canNext: boolean;
  canPrev: boolean;
  canSeek: boolean;
  shuffle: boolean | null;
  repeat: 'none' | 'track' | 'list' | null;
}

export interface MediaState {
  sessions: SmtcSession[];
  current: string | null;
}

export interface Activity {
  verb: string;
  target: string;
  tool: string;
  kind: 'run' | 'read' | 'edit' | 'search' | 'web' | 'agent' | 'plan' | 'tool' | 'think';
}

export interface AgentLog {
  at: number;
  kind: string;
  text: string;
}

export interface AgentSession {
  id: string;
  agent: 'claude' | 'codex';
  project: string;
  cwd: string;
  status: 'idle' | 'working' | 'waiting' | 'done' | 'error' | 'ended';
  activity: Activity | null;
  prompt: string;
  message: string;
  turnStartedAt: number;
  updatedAt: number;
  tools: number;
  edited: string[];
  subagents: number;
  log: AgentLog[];
}

export interface Approval {
  id: string;
  sessionId: string;
  project: string;
  tool: string;
  title: string;
  detail: string;
  createdAt: number;
  expiresAt: number;
}

export interface AgentsSnapshot {
  sessions: AgentSession[];
  approvals: Approval[];
}

export interface AgentDone {
  sessionId: string;
  project: string;
  message: string;
  durationMs: number;
  tools: number;
  edited: string[];
}

export interface QuotaItem {
  label: string;
  percent: number;
  resetsAt: number | null;
  windowMinutes: number | null;
}

export interface QuotaProvider {
  provider: 'claude' | 'codex';
  ok: boolean;
  plan: string;
  error: string;
  items: QuotaItem[];
  fetchedAt: number;
}

export interface Battery {
  percent: number;
  charging: boolean;
  plugged: boolean;
  saver: boolean;
  secondsLeft: number | null;
}

export interface Stats {
  cpu: number;
  memory: number;
  memoryUsedGb: number;
  memoryTotalGb: number;
  netDown: number;
  netUp: number;
  battery: Battery | null;
}

export interface Clip {
  id: number;
  text: string;
  at: number;
}

export interface Radios {
  allowed: boolean;
  wifi: boolean | null;
  bluetooth: boolean | null;
  ssid: string | null;
  signal: number | null;
}

export interface Volume {
  level: number;
  muted: boolean;
}

export interface HookStatus {
  installed: boolean;
  events: number;
  path: string;
  claudeFound: boolean;
  port: number;
  mode: string;
}

export interface MonitorInfo {
  index: number;
  name: string;
  primary: boolean;
  width: number;
  height: number;
  scale: number;
}

/** A game in front (games.rs). */
export interface Game {
  name: string;
  exe: string;
}

/** A game that closed, and how long it ran. */
export interface GameEnded {
  name: string;
  exe: string;
  durationMs: number;
}

/** The island window right now (island.rs `island_state`). */
/** An AI coding tool found on this PC (harness_scan). */
export interface Tool {
  id: string;
  name: string;
  installed: boolean;
  running: boolean;
  lastUsed: number | null;
  today: ToolToday | null;
  /** Has plan limits (usage rings): Claude Code and Codex. */
  plan: boolean;
}

export interface ToolToday {
  /** Tokens sent and received today, cache reads left out; null when the tool does not log them. */
  tokens: number | null;
  sessions: number;
  /** US dollars the tool itself recorded (opencode). */
  cost: number | null;
}

/** A maximized window in front of the island's monitor and its program ("vivaldi", "T3 Code"). */
export interface Front {
  maximized: boolean;
  app: string;
}

export interface IslandState {
  maximized: boolean;
  app: string;
  hidden: boolean;
  shown: boolean;
}

/** `update_check`: the running version against the latest GitHub release. */
export interface UpdateInfo {
  current: string;
  latest: string;
  available: boolean;
  /** The release page, for "What's new". */
  url: string;
  /** An installed copy, which can update itself (the portable exe can't). */
  installed: boolean;
}

/** `update://progress` while the installer downloads, in bytes. */
export interface UpdateProgress {
  received: number;
  total: number;
}

/** `uninstall_info`. */
export interface UninstallInfo {
  installed: boolean;
  dataDir: string;
}
