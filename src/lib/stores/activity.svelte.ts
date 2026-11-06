// Short-lived "peeks": the island grows for a moment to say something (an agent finished,
// a track changed, the volume moved) and settles back. Higher priority wins; equal priority
// replaces. Approvals are not peeks: they stay until answered (see IslandApp).

export type PeekKind = 'done' | 'attention' | 'track' | 'volume' | 'charging' | 'timer' | 'shelf' | 'pear' | 'game' | 'gameover' | 'missed';

export interface Peek {
  id: number;
  kind: PeekKind;
  data: Record<string, unknown>;
  until: number;
  priority: number;
}

const PRIORITY: Record<PeekKind, number> = {
  game: 100,
  attention: 90,
  missed: 85,
  gameover: 75,
  done: 80,
  timer: 70,
  shelf: 60,
  pear: 55,
  charging: 50,
  track: 30,
  volume: 20,
};

/** Pop-ups that are only nice to have: dropped while Do not disturb is on. */
const CHATTY: PeekKind[] = ['track', 'volume', 'charging', 'pear'];

class ActivityStore {
  peek = $state<Peek | null>(null);
  /** Windows' Do not disturb is on and the user wants it respected. */
  quiet = false;
  private seq = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;

  show(kind: PeekKind, data: Record<string, unknown> = {}, ms = 4000): void {
    if (this.quiet && CHATTY.includes(kind)) return;
    const priority = PRIORITY[kind];
    const current = this.peek;
    if (current && current.priority > priority && current.until > Date.now()) return;
    // Volume steps arrive in bursts: keep the same peek and extend it instead of replacing.
    const id = current && current.kind === kind && kind === 'volume' ? current.id : ++this.seq;
    this.peek = { id, kind, data, until: Date.now() + ms, priority };
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => this.dismiss(), ms);
  }

  dismiss(): void {
    if (this.timer) clearTimeout(this.timer);
    this.peek = null;
  }
}

export const activity = new ActivityStore();
