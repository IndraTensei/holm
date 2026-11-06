// Whether the island should rest as a thin line over the maximized window in front, learned
// per app from how the island gets used there.
//
// An open on hover that closes again within 2 s with no click, scroll or key inside is a miss:
// the island was in the way (a browser tab, a title bar button). Three misses over one app
// within three days, with no real use in between, and the island rests as a line over that app
// only; hover still opens it. Real use over an app clears its misses. Settings → General lists
// the learned apps and can undo each one. Browsers start as a line (their tabs sit right under
// the island), unless "browsersAsLine" is off.
//
// While resting as a line over a maximized app, only the very top edge of the screen opens the
// island (see IslandApp's reportRect), so tabs and title bar buttons under it stay clickable.
import { call, send } from '../bridge';
import { settings } from './settings.svelte';
import { activity } from './activity.svelte';
import type { Front } from '../types';

const QUICK_MS = 2000;
const WINDOW_MS = 3 * 24 * 3600_000;
const STRIKES = 3;
const SAME_MS = 1500;
const BROWSERS = ['vivaldi', 'chrome', 'msedge', 'firefox', 'brave', 'opera', 'opera_gx', 'arc', 'zen', 'librewolf', 'waterfox', 'floorp', 'thorium', 'chromium', 'helium', 'comet', 'dia'];

/** "T3 Code" → "t3 code": the key in `lineApps` and the misses. */
export const appKey = (app: string) => app.trim().toLowerCase();

class Fit {
  maximized = $state(false);
  app = $state('');
  private misses: Record<string, number[]> = {};
  private trial: { app: string; at: number; used: boolean } | null = null;

  async load(): Promise<void> {
    this.misses = (await call<Record<string, number[]> | null>('data_get', { key: 'lineMisses' }).catch(() => null)) ?? {};
  }

  set(front: Front): void {
    this.maximized = front.maximized;
    this.app = front.app;
  }

  /** The island should rest as a line right now. */
  get line(): boolean {
    const mode = settings.v.overMaximized;
    if (!this.maximized || mode === 'full') return false;
    const key = appKey(this.app);
    return mode === 'line' || settings.v.lineApps.includes(key) || (settings.v.browsersAsLine && BROWSERS.includes(key));
  }

  opened(by: 'hover' | 'click' | 'hotkey'): void {
    const learning = settings.v.overMaximized === 'learn' && this.maximized && this.app && !this.line;
    this.trial = by === 'hover' && learning ? { app: this.app, at: Date.now(), used: false } : null;
  }

  /** A click, scroll or key inside the open island. */
  used(): void {
    if (this.trial) this.trial.used = true;
  }

  closed(): void {
    const t = this.trial;
    this.trial = null;
    if (!t) return;
    const key = appKey(t.app);
    if (t.used) {
      if (this.misses[key]) {
        delete this.misses[key];
        this.save();
      }
      return;
    }
    const now = Date.now();
    if (now - t.at > QUICK_MS) return;
    const recent = (this.misses[key] ?? []).filter((at) => now - at < WINDOW_MS);
    // The hover signal can flicker while the island grows: one accident, counted once.
    if (recent.length && now - recent[recent.length - 1] < SAME_MS) return;
    recent.push(now);
    if (recent.length >= STRIKES) {
      delete this.misses[key];
      void settings.set({ lineApps: [...settings.v.lineApps.filter((a) => a !== key), key] });
      activity.show('note', { title: `Resting as a line over ${t.app}`, sub: 'It kept opening by accident there. Point at the line to open it.' }, 6000);
    } else {
      this.misses[key] = recent;
    }
    this.save();
  }

  private save(): void {
    send('data_set', { key: 'lineMisses', value: this.misses });
  }
}

export const fit = new Fit();
