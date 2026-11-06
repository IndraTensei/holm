// AI coding tools found on this PC and what each used today (Rust: harness.rs). Scanned only
// while something that shows them is on screen: the Agents page or Settings → Harnesses.
import { call } from '../bridge';
import { settings } from './settings.svelte';
import type { Tool } from '../types';

const EVERY_MS = 5 * 60_000;

function midnight(): number {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

/** Shown on the Agents page: Claude Code and Codex by default (when on this PC), others when turned on. */
export function shown(id: string): boolean {
  const s = settings.v;
  if (id === 'claude') return s.quotaClaude;
  if (id === 'codex') return s.quotaCodex;
  return s.toolsShown[id] ?? false;
}

export function setShown(id: string, on: boolean): void {
  if (id === 'claude') void settings.set({ quotaClaude: on });
  else if (id === 'codex') void settings.set({ quotaCodex: on });
  else void settings.set({ toolsShown: { ...settings.v.toolsShown, [id]: on } });
}

class Tools {
  list = $state<Tool[]>([]);
  scannedAt = $state(0);
  scanning = $state(false);
  private users = 0;
  private timer: ReturnType<typeof setInterval> | null = null;

  installed = $derived(this.list.filter((t) => t.installed));

  async scan(): Promise<void> {
    if (this.scanning) return;
    this.scanning = true;
    try {
      this.list = (await call<Tool[]>('harness_scan', { since: midnight() }).catch(() => null)) ?? this.list;
      this.scannedAt = Date.now();
    } finally {
      this.scanning = false;
    }
  }

  /** Call from an $effect while tools are on screen; returns the release function. */
  use(): () => void {
    if (this.users++ === 0) {
      if (Date.now() - this.scannedAt > 30_000) void this.scan();
      this.timer = setInterval(() => void this.scan(), EVERY_MS);
    }
    return () => {
      if (--this.users === 0 && this.timer) {
        clearInterval(this.timer);
        this.timer = null;
      }
    };
  }
}

export const tools = new Tools();
